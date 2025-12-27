import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, notFound, serverError } from '@/lib/auth'
import { estimateCost } from '@/lib/utils'

// GET /api/sessions/:id - Get session details
export async function GET(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  try {
    const session = await prisma.session.findFirst({
      where: {
        id: params.id,
        userId: user.id,
      },
      include: {
        project: true,
        memories: {
          orderBy: { createdAt: 'desc' },
        },
      },
    })

    if (!session) {
      return notFound('Session not found')
    }

    return Response.json(session)
  } catch (error) {
    console.error('Get session error:', error)
    return serverError('Failed to get session')
  }
}

// PUT /api/sessions/:id - Update session (end, add tokens)
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
    const { tokensIn, tokensOut, model, status, end } = body

    // Find the session first
    const existing = await prisma.session.findFirst({
      where: {
        id: params.id,
        userId: user.id,
      },
    })

    if (!existing) {
      return notFound('Session not found')
    }

    // Calculate cost if token info provided
    const newTokensIn = existing.tokensIn + (tokensIn || 0)
    const newTokensOut = existing.tokensOut + (tokensOut || 0)
    const modelToUse = model || existing.model || 'gpt-4o'
    const costUsd = estimateCost(modelToUse, newTokensIn, newTokensOut)

    const session = await prisma.session.update({
      where: { id: params.id },
      data: {
        tokensIn: newTokensIn,
        tokensOut: newTokensOut,
        costUsd,
        ...(model && { model }),
        ...(status && { status }),
        ...(end && { endedAt: new Date(), status: 'completed' }),
      },
    })

    return Response.json(session)
  } catch (error) {
    console.error('Update session error:', error)
    return serverError('Failed to update session')
  }
}
