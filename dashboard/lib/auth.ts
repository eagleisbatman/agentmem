import { prisma } from './prisma'
import { NextRequest } from 'next/server'

export interface AuthUser {
  id: string
  email: string
  name: string | null
  apiKey: string
}

export async function authenticateRequest(request: NextRequest): Promise<AuthUser | null> {
  const authHeader = request.headers.get('authorization')

  if (!authHeader) {
    return null
  }

  // Support both "Bearer <token>" and just "<token>"
  const apiKey = authHeader.startsWith('Bearer ')
    ? authHeader.slice(7)
    : authHeader

  if (!apiKey || !apiKey.startsWith('am_')) {
    return null
  }

  try {
    const user = await prisma.user.findUnique({
      where: { apiKey },
      select: {
        id: true,
        email: true,
        name: true,
        apiKey: true,
      },
    })

    return user
  } catch (error) {
    console.error('Auth error:', error)
    return null
  }
}

export function unauthorized() {
  return Response.json(
    { error: 'Unauthorized', message: 'Invalid or missing API key' },
    { status: 401 }
  )
}

export function badRequest(message: string) {
  return Response.json(
    { error: 'Bad Request', message },
    { status: 400 }
  )
}

export function notFound(message: string = 'Resource not found') {
  return Response.json(
    { error: 'Not Found', message },
    { status: 404 }
  )
}

export function serverError(message: string = 'Internal server error') {
  return Response.json(
    { error: 'Server Error', message },
    { status: 500 }
  )
}
