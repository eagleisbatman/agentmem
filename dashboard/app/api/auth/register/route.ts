import { NextRequest } from 'next/server'
import { prisma } from '@/lib/prisma'
import { generateApiKey } from '@/lib/utils'
import { badRequest, serverError } from '@/lib/auth'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { email, name } = body

    if (!email) {
      return badRequest('Email is required')
    }

    // Check if user already exists
    const existing = await prisma.user.findUnique({
      where: { email },
    })

    if (existing) {
      return badRequest('User with this email already exists')
    }

    // Create new user with API key
    const apiKey = generateApiKey()
    const user = await prisma.user.create({
      data: {
        email,
        name: name || null,
        apiKey,
      },
    })

    return Response.json({
      id: user.id,
      email: user.email,
      name: user.name,
      apiKey: user.apiKey,
      message: 'Account created successfully. Save your API key - it will only be shown once.',
    })
  } catch (error) {
    console.error('Registration error:', error)
    return serverError('Failed to create account')
  }
}
