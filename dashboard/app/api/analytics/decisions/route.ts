import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { authenticateRequest, unauthorized, serverError } from '@/lib/auth'

// GET /api/analytics/decisions - Decision timeline
export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  const { searchParams } = new URL(request.url)
  const period = searchParams.get('period') || '7d'
  const projectId = searchParams.get('projectId')
  const outcome = searchParams.get('outcome') // success, failed, or all

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
    // Get memories (decisions, corrections, etc.) in the time range
    const memories = await prisma.memory.findMany({
      where: {
        userId: user.id,
        createdAt: { gte: startDate },
        ...(projectId && { projectId }),
        ...(outcome && outcome !== 'all' && { outcome }),
      },
      include: {
        project: {
          select: { name: true },
        },
      },
      orderBy: { createdAt: 'desc' },
      take: 100,
    })

    // Group by date for timeline
    const timeline: Record<string, typeof memories> = {}
    memories.forEach(memory => {
      const dateKey = memory.createdAt.toISOString().split('T')[0]
      if (!timeline[dateKey]) {
        timeline[dateKey] = []
      }
      timeline[dateKey].push(memory)
    })

    // Calculate stats
    const stats = {
      total: memories.length,
      byType: {} as Record<string, number>,
      byOutcome: {
        success: 0,
        failed: 0,
        unknown: 0,
      },
      byAgent: {} as Record<string, number>,
    }

    memories.forEach(memory => {
      // By type
      stats.byType[memory.memoryType] = (stats.byType[memory.memoryType] || 0) + 1

      // By outcome
      if (memory.outcome === 'success') stats.byOutcome.success++
      else if (memory.outcome === 'failed') stats.byOutcome.failed++
      else stats.byOutcome.unknown++

      // By agent
      const agent = memory.agent || 'unknown'
      stats.byAgent[agent] = (stats.byAgent[agent] || 0) + 1
    })

    const successRate = stats.total > 0
      ? ((stats.byOutcome.success / (stats.byOutcome.success + stats.byOutcome.failed || 1)) * 100).toFixed(1)
      : '0'

    return Response.json({
      period,
      startDate: startDate.toISOString(),
      endDate: now.toISOString(),
      stats: {
        ...stats,
        successRate: parseFloat(successRate),
      },
      timeline: Object.entries(timeline).map(([date, items]) => ({
        date,
        items,
      })),
      memories, // Raw list for detailed view
    })
  } catch (error) {
    console.error('Decision analytics error:', error)
    return serverError('Failed to get decision analytics')
  }
}
