import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, badRequest, serverError } from '@/lib/auth'

// GET /api/projects - List all projects
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const projects = await prisma.project.findMany({
      where: { userId: user.id },
      include: {
        _count: {
          select: {
            memories: true,
            sessions: true,
            tasks: true,
          },
        },
      },
      orderBy: { lastActiveAt: 'desc' },
    })

    return Response.json(projects)
  } catch (error) {
    console.error('List projects error:', error)
    return serverError('Failed to list projects')
  }
}

// POST /api/projects - Register a project
export async function POST(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const body = await request.json()
    const { name, path, machineId } = body

    if (!name) {
      return badRequest('Project name is required')
    }

    // Upsert - create or update lastActiveAt
    const project = await prisma.project.upsert({
      where: {
        userId_name_machineId: {
          userId: user.id,
          name,
          machineId: machineId || null,
        },
      },
      update: {
        lastActiveAt: new Date(),
        path: path || undefined,
      },
      create: {
        userId: user.id,
        name,
        path: path || null,
        machineId: machineId || null,
      },
    })

    return Response.json(project)
  } catch (error) {
    console.error('Create project error:', error)
    return serverError('Failed to create project')
  }
}
