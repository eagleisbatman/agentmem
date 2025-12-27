import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, badRequest, serverError } from '@/lib/auth'

// GET /api/memories - List memories
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  const { searchParams } = new URL(request.url)
  const projectId = searchParams.get('projectId')
  const scope = searchParams.get('scope') // "global", "project", or null for all
  const memoryType = searchParams.get('type')
  const limit = parseInt(searchParams.get('limit') || '100')
  const outcome = searchParams.get('outcome')

  try {
    const memories = await prisma.memory.findMany({
      where: {
        userId: user.id,
        ...(projectId && { projectId }),
        ...(scope && { scope }),
        ...(memoryType && { memoryType }),
        ...(outcome && { outcome }),
      },
      include: {
        project: {
          select: { name: true },
        },
        session: {
          select: { agent: true, model: true },
        },
      },
      orderBy: { createdAt: 'desc' },
      take: limit,
    })

    return Response.json(memories)
  } catch (error) {
    console.error('List memories error:', error)
    return serverError('Failed to list memories')
  }
}

// POST /api/memories - Create memory
export async function POST(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const body = await request.json()
    const {
      projectId,
      projectName,
      sessionId,
      scope = 'project',
      memoryType,
      title,
      content,
      agent,
      model,
      confidence = 70,
      source,
      machineId,
    } = body

    if (!memoryType || !title) {
      return badRequest('Memory type and title are required')
    }

    // Resolve project ID if project name is provided
    let resolvedProjectId = projectId
    if (!resolvedProjectId && projectName && scope === 'project') {
      const project = await prisma.project.upsert({
        where: {
          userId_name_machineId: {
            userId: user.id,
            name: projectName,
            machineId: machineId || null,
          },
        },
        update: { lastActiveAt: new Date() },
        create: {
          userId: user.id,
          name: projectName,
          machineId: machineId || null,
        },
      })
      resolvedProjectId = project.id
    }

    // For project scope, we need a project ID
    if (scope === 'project' && !resolvedProjectId) {
      return badRequest('Project ID or project name is required for project-scoped memories')
    }

    const memory = await prisma.memory.create({
      data: {
        userId: user.id,
        projectId: scope === 'global' ? null : resolvedProjectId,
        sessionId: sessionId || null,
        scope,
        memoryType,
        title,
        content: content || null,
        agent: agent || null,
        model: model || null,
        confidence,
        source: source || null,
      },
    })

    return Response.json(memory)
  } catch (error) {
    console.error('Create memory error:', error)
    return serverError('Failed to create memory')
  }
}
