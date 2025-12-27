import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, notFound, serverError } from '@/lib/auth'

// GET /api/memories/:id - Get memory details
export async function GET(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const memory = await prisma.memory.findFirst({
      where: {
        id: params.id,
        userId: user.id,
      },
      include: {
        project: true,
        session: true,
      },
    })

    if (!memory) {
      return notFound('Memory not found')
    }

    return Response.json(memory)
  } catch (error) {
    console.error('Get memory error:', error)
    return serverError('Failed to get memory')
  }
}

// PUT /api/memories/:id - Update memory
export async function PUT(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const body = await request.json()
    const { title, content, outcome, confidence } = body

    const existing = await prisma.memory.findFirst({
      where: {
        id: params.id,
        userId: user.id,
      },
    })

    if (!existing) {
      return notFound('Memory not found')
    }

    const memory = await prisma.memory.update({
      where: { id: params.id },
      data: {
        ...(title !== undefined && { title }),
        ...(content !== undefined && { content }),
        ...(outcome !== undefined && { outcome }),
        ...(confidence !== undefined && { confidence }),
      },
    })

    return Response.json(memory)
  } catch (error) {
    console.error('Update memory error:', error)
    return serverError('Failed to update memory')
  }
}

// DELETE /api/memories/:id - Delete memory
export async function DELETE(
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

    await prisma.memory.delete({
      where: { id: params.id },
    })

    return Response.json({ success: true })
  } catch (error) {
    console.error('Delete memory error:', error)
    return serverError('Failed to delete memory')
  }
}
