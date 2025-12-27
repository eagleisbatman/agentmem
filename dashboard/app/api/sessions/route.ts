import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, badRequest, serverError } from '@/lib/auth'

// GET /api/sessions - List sessions
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  const { searchParams } = new URL(request.url)
  const projectId = searchParams.get('projectId')
  const limit = parseInt(searchParams.get('limit') || '50')
  const agent = searchParams.get('agent')

  try {
    const sessions = await prisma.session.findMany({
      where: {
        userId: user.id,
        ...(projectId && { projectId }),
        ...(agent && { agent }),
      },
      include: {
        project: {
          select: { name: true },
        },
        _count: {
          select: { memories: true },
        },
      },
      orderBy: { startedAt: 'desc' },
      take: limit,
    })

    return Response.json(sessions)
  } catch (error) {
    console.error('List sessions error:', error)
    return serverError('Failed to list sessions')
  }
}

// POST /api/sessions - Start a new session
export async function POST(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const body = await request.json()
    const { projectId, projectName, agent, model, machineId } = body

    if (!agent) {
      return badRequest('Agent is required')
    }

    // If projectName is provided but not projectId, look up or create the project
    let resolvedProjectId = projectId
    if (!resolvedProjectId && projectName) {
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

    if (!resolvedProjectId) {
      return badRequest('Project ID or project name is required')
    }

    const session = await prisma.session.create({
      data: {
        userId: user.id,
        projectId: resolvedProjectId,
        agent,
        model: model || null,
        status: 'active',
      },
    })

    return Response.json(session)
  } catch (error) {
    console.error('Create session error:', error)
    return serverError('Failed to create session')
  }
}
