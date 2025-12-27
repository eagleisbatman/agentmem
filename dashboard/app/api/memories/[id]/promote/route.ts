import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, notFound, badRequest, serverError } from '@/lib/auth'

// POST /api/memories/:id/promote - Promote memory to global scope
export async function POST(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const existing = await prisma.memory.findFirst({
      where: {
        id: params.id,
        userId: user.id,
      },
    })

    if (!existing) {
      return notFound('Memory not found')
    }

    if (existing.scope === 'global') {
      return badRequest('Memory is already global')
    }

    const memory = await prisma.memory.update({
      where: { id: params.id },
      data: {
        scope: 'global',
        projectId: null,
      },
    })

    return Response.json(memory)
  } catch (error) {
    console.error('Promote memory error:', error)
    return serverError('Failed to promote memory')
  }
}
