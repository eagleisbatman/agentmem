import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, badRequest, serverError } from '@/lib/auth'

// GET /api/context - Get context for prompt injection
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  const { searchParams } = new URL(request.url)
  const projectName = searchParams.get('project')
  const query = searchParams.get('query')
  const limit = parseInt(searchParams.get('limit') || '10')
  const machineId = searchParams.get('machineId')

  if (!projectName) {
    return badRequest('Project name is required')
  }

  try {
    // Find the project
    const project = await prisma.project.findFirst({
      where: {
        userId: user.id,
        name: projectName,
        ...(machineId && { machineId }),
      },
    })

    // Get global memories
    const globalMemories = await prisma.memory.findMany({
      where: {
        userId: user.id,
        scope: 'global',
      },
      orderBy: { lastObservedAt: 'desc' },
      take: limit,
    })

    // Get project-specific memories
    let projectMemories: typeof globalMemories = []
    let protectedFiles: { pattern: string; reason: string | null }[] = []
    let tasks: { id: string; title: string; status: string; priority: number }[] = []

    if (project) {
      // If there's a query, try to filter by it
      const searchCondition = query
        ? {
            OR: [
              { title: { contains: query, mode: 'insensitive' as const } },
              { content: { contains: query, mode: 'insensitive' as const } },
            ],
          }
        : {}

      projectMemories = await prisma.memory.findMany({
        where: {
          userId: user.id,
          projectId: project.id,
          scope: 'project',
          ...searchCondition,
        },
        orderBy: { lastObservedAt: 'desc' },
        take: limit,
      })

      protectedFiles = await prisma.protectedFile.findMany({
        where: {
          userId: user.id,
          projectId: project.id,
        },
        select: {
          pattern: true,
          reason: true,
        },
      })

      tasks = await prisma.task.findMany({
        where: {
          userId: user.id,
          projectId: project.id,
          status: { not: 'done' },
        },
        select: {
          id: true,
          title: true,
          status: true,
          priority: true,
        },
        orderBy: { priority: 'asc' },
        take: 10,
      })

      // Update last active
      await prisma.project.update({
        where: { id: project.id },
        data: { lastActiveAt: new Date() },
      })
    }

    // Combine and deduplicate memories
    const allMemories = [...globalMemories, ...projectMemories]
    const uniqueMemories = allMemories.filter(
      (m, i, arr) => arr.findIndex(x => x.id === m.id) === i
    )

    return Response.json({
      project: project?.name || projectName,
      projectId: project?.id || null,
      memories: uniqueMemories.map(m => ({
        id: m.id,
        memory_type: m.memoryType,
        title: m.title,
        content: m.content,
        scope: m.scope,
        agent: m.agent,
        model: m.model,
      })),
      protected: protectedFiles,
      tasks: tasks.map(t => ({
        id: t.id,
        title: t.title,
        status: t.status,
        priority: t.priority,
      })),
      tools: [], // Reserved for future tool registry
    })
  } catch (error) {
    console.error('Context error:', error)
    return serverError('Failed to get context')
  }
}
