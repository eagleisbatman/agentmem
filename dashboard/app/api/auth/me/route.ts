import { NextRequest } from 'next/server'
import { authenticateRequest, unauthorized } from '@/lib/auth'
import { prisma } from '@/lib/prisma'

export async function GET(request: NextRequest) {
  const user = await authenticateRequest(request)
  if (!user) {
    return unauthorized()
  }

  // Get user stats
  const [projectCount, memoryCount, sessionCount] = await Promise.all([
    prisma.project.count({ where: { userId: user.id } }),
    prisma.memory.count({ where: { userId: user.id } }),
    prisma.session.count({ where: { userId: user.id } }),
  ])

  return Response.json({
    id: user.id,
    email: user.email,
    name: user.name,
    stats: {
      projects: projectCount,
      memories: memoryCount,
      sessions: sessionCount,
    },
  })
}
