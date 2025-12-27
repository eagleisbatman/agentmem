import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, serverError } from '@/lib/auth'

// GET /api/analytics/tokens - Token usage over time
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  const { searchParams } = new URL(request.url)
  const period = searchParams.get('period') || '7d'
  const projectId = searchParams.get('projectId')

  // Calculate date range
  const now = new Date()
  let startDate: Date
  switch (period) {
    case '30d':
      startDate = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000)
      break
    case '90d':
      startDate = new Date(now.getTime() - 90 * 24 * 60 * 60 * 1000)
      break
    default: // 7d
      startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
  }

  try {
    // Get sessions in the time range
    const sessions = await prisma.session.findMany({
      where: {
        userId: user.id,
        startedAt: { gte: startDate },
        ...(projectId && { projectId }),
      },
      select: {
        startedAt: true,
        tokensIn: true,
        tokensOut: true,
        costUsd: true,
        agent: true,
        model: true,
      },
      orderBy: { startedAt: 'asc' },
    })

    // Aggregate by day
    const dailyStats: Record<string, {
      date: string
      tokensIn: number
      tokensOut: number
      cost: number
      sessions: number
    }> = {}

    sessions.forEach(session => {
      const dateKey = session.startedAt.toISOString().split('T')[0]
      if (!dailyStats[dateKey]) {
        dailyStats[dateKey] = {
          date: dateKey,
          tokensIn: 0,
          tokensOut: 0,
          cost: 0,
          sessions: 0,
        }
      }
      dailyStats[dateKey].tokensIn += session.tokensIn
      dailyStats[dateKey].tokensOut += session.tokensOut
      dailyStats[dateKey].cost += Number(session.costUsd)
      dailyStats[dateKey].sessions += 1
    })

    // Calculate totals
    const totals = {
      tokensIn: sessions.reduce((sum, s) => sum + s.tokensIn, 0),
      tokensOut: sessions.reduce((sum, s) => sum + s.tokensOut, 0),
      cost: sessions.reduce((sum, s) => sum + Number(s.costUsd), 0),
      sessions: sessions.length,
    }

    // Agent breakdown
    const agentStats: Record<string, {
      tokensIn: number
      tokensOut: number
      cost: number
      sessions: number
    }> = {}

    sessions.forEach(session => {
      if (!agentStats[session.agent]) {
        agentStats[session.agent] = {
          tokensIn: 0,
          tokensOut: 0,
          cost: 0,
          sessions: 0,
        }
      }
      agentStats[session.agent].tokensIn += session.tokensIn
      agentStats[session.agent].tokensOut += session.tokensOut
      agentStats[session.agent].cost += Number(session.costUsd)
      agentStats[session.agent].sessions += 1
    })

    // Model breakdown
    const modelStats: Record<string, {
      tokensIn: number
      tokensOut: number
      cost: number
      sessions: number
    }> = {}

    sessions.forEach(session => {
      const model = session.model || 'unknown'
      if (!modelStats[model]) {
        modelStats[model] = {
          tokensIn: 0,
          tokensOut: 0,
          cost: 0,
          sessions: 0,
        }
      }
      modelStats[model].tokensIn += session.tokensIn
      modelStats[model].tokensOut += session.tokensOut
      modelStats[model].cost += Number(session.costUsd)
      modelStats[model].sessions += 1
    })

    return Response.json({
      period,
      startDate: startDate.toISOString(),
      endDate: now.toISOString(),
      totals,
      daily: Object.values(dailyStats),
      byAgent: agentStats,
      byModel: modelStats,
    })
  } catch (error) {
    console.error('Token analytics error:', error)
    return serverError('Failed to get token analytics')
  }
}
