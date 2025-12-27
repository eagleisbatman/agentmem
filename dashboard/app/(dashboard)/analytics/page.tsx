'use client'

import { useState } from 'react'
import {
  BarChart3,
  TrendingUp,
  DollarSign,
  Zap,
  CheckCircle,
  XCircle,
  Clock,
} from 'lucide-react'

// Placeholder data
const tokenData = {
  daily: [
    { date: 'Mon', tokensIn: 45000, tokensOut: 12000 },
    { date: 'Tue', tokensIn: 68000, tokensOut: 18000 },
    { date: 'Wed', tokensIn: 120000, tokensOut: 35000 },
    { date: 'Thu', tokensIn: 89000, tokensOut: 24000 },
    { date: 'Fri', tokensIn: 95000, tokensOut: 28000 },
    { date: 'Sat', tokensIn: 34000, tokensOut: 9000 },
    { date: 'Sun', tokensIn: 150000, tokensOut: 42000 },
  ],
  totals: {
    tokensIn: 601000,
    tokensOut: 168000,
    cost: 4.23,
    sessions: 23,
  },
}

const agentBreakdown = [
  { agent: 'Claude Code', tokens: 568000, percentage: 67, sessions: 15 },
  { agent: 'Cursor', tokens: 237000, percentage: 28, sessions: 6 },
  { agent: 'Gemini CLI', tokens: 42000, percentage: 5, sessions: 2 },
]

const modelBreakdown = [
  { model: 'claude-sonnet-4-20250514', tokens: 498000, percentage: 59, cost: 2.49 },
  { model: 'gpt-4o', tokens: 198000, percentage: 23, cost: 0.99 },
  { model: 'claude-opus-4-20250514', tokens: 112000, percentage: 13, cost: 1.68 },
  { model: 'gemini-2.0-flash', tokens: 42000, percentage: 5, cost: 0.003 },
]

const decisionTimeline = [
  {
    date: 'Today',
    items: [
      { time: '14:32', type: 'decision', title: 'Use MDC format for Cursor', outcome: 'success', agent: 'Claude' },
      { time: '14:28', type: 'decision', title: 'Wrapper script for Codex', outcome: 'success', agent: 'Claude' },
      { time: '11:45', type: 'correction', title: 'Fixed Gemini settings', outcome: 'failed', agent: 'Claude' },
      { time: '10:30', type: 'decision', title: 'Multi-agent hook system', outcome: 'success', agent: 'Claude' },
    ],
  },
  {
    date: 'Yesterday',
    items: [
      { time: '18:22', type: 'decision', title: 'Use Qdrant for vectors', outcome: 'success', agent: 'Cursor' },
      { time: '15:10', type: 'gotcha', title: 'Qdrant API deprecated', outcome: 'failed', agent: 'Claude' },
    ],
  },
]

const periods = [
  { value: '7d', label: '7 Days' },
  { value: '30d', label: '30 Days' },
  { value: '90d', label: '90 Days' },
]

export default function AnalyticsPage() {
  const [period, setPeriod] = useState('7d')

  const maxTokens = Math.max(...tokenData.daily.map(d => d.tokensIn + d.tokensOut))

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Analytics</h1>
          <p className="text-muted-foreground">
            Token usage, costs, and decision outcomes
          </p>
        </div>
        <select
          value={period}
          onChange={(e) => setPeriod(e.target.value)}
          className="rounded-md border border-border bg-background px-3 py-2 text-sm focus:border-foreground focus:outline-none"
        >
          {periods.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
      </div>

      {/* Summary Stats */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center gap-2 text-muted-foreground">
            <Zap className="h-4 w-4" />
            <span className="text-sm">Total Tokens</span>
          </div>
          <p className="mt-2 text-3xl font-bold">
            {((tokenData.totals.tokensIn + tokenData.totals.tokensOut) / 1000).toFixed(0)}k
          </p>
          <p className="text-xs text-muted-foreground">
            {(tokenData.totals.tokensIn / 1000).toFixed(0)}k in / {(tokenData.totals.tokensOut / 1000).toFixed(0)}k out
          </p>
        </div>
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center gap-2 text-muted-foreground">
            <DollarSign className="h-4 w-4" />
            <span className="text-sm">Estimated Cost</span>
          </div>
          <p className="mt-2 text-3xl font-bold">${tokenData.totals.cost.toFixed(2)}</p>
          <p className="text-xs text-muted-foreground">This period</p>
        </div>
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center gap-2 text-muted-foreground">
            <Clock className="h-4 w-4" />
            <span className="text-sm">Sessions</span>
          </div>
          <p className="mt-2 text-3xl font-bold">{tokenData.totals.sessions}</p>
          <p className="text-xs text-muted-foreground">Across all agents</p>
        </div>
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center gap-2 text-muted-foreground">
            <TrendingUp className="h-4 w-4" />
            <span className="text-sm">Avg per Session</span>
          </div>
          <p className="mt-2 text-3xl font-bold">
            {((tokenData.totals.tokensIn + tokenData.totals.tokensOut) / tokenData.totals.sessions / 1000).toFixed(1)}k
          </p>
          <p className="text-xs text-muted-foreground">tokens/session</p>
        </div>
      </div>

      {/* Token Usage Chart */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold">Token Usage</h2>
        <p className="text-sm text-muted-foreground">Daily breakdown</p>

        <div className="mt-6">
          {/* Simple bar chart */}
          <div className="flex items-end gap-2 h-48">
            {tokenData.daily.map((day, i) => {
              const height = ((day.tokensIn + day.tokensOut) / maxTokens) * 100
              const inHeight = (day.tokensIn / (day.tokensIn + day.tokensOut)) * 100
              return (
                <div key={i} className="flex-1 flex flex-col items-center gap-2">
                  <div
                    className="w-full rounded-t relative overflow-hidden"
                    style={{ height: `${height}%` }}
                  >
                    <div
                      className="absolute bottom-0 w-full bg-foreground/80"
                      style={{ height: `${inHeight}%` }}
                    />
                    <div
                      className="absolute top-0 w-full bg-foreground/30"
                      style={{ height: `${100 - inHeight}%` }}
                    />
                  </div>
                  <span className="text-xs text-muted-foreground">{day.date}</span>
                </div>
              )
            })}
          </div>
          <div className="mt-4 flex items-center gap-6 text-xs">
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-foreground/80" />
              <span className="text-muted-foreground">Input tokens</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-foreground/30" />
              <span className="text-muted-foreground">Output tokens</span>
            </div>
          </div>
        </div>
      </div>

      <div className="grid gap-8 lg:grid-cols-2">
        {/* Agent Breakdown */}
        <div className="rounded-lg border border-border bg-card p-6">
          <h2 className="text-lg font-semibold">By Agent</h2>
          <div className="mt-4 space-y-4">
            {agentBreakdown.map((item) => (
              <div key={item.agent}>
                <div className="flex items-center justify-between text-sm">
                  <span>{item.agent}</span>
                  <span className="text-muted-foreground">
                    {(item.tokens / 1000).toFixed(0)}k tokens ({item.sessions} sessions)
                  </span>
                </div>
                <div className="mt-2 h-2 rounded-full bg-secondary">
                  <div
                    className="h-full rounded-full bg-foreground"
                    style={{ width: `${item.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Model Breakdown */}
        <div className="rounded-lg border border-border bg-card p-6">
          <h2 className="text-lg font-semibold">By Model</h2>
          <div className="mt-4 space-y-4">
            {modelBreakdown.map((item) => (
              <div key={item.model}>
                <div className="flex items-center justify-between text-sm">
                  <span className="font-mono text-xs">{item.model}</span>
                  <span className="text-muted-foreground">
                    ${item.cost.toFixed(2)}
                  </span>
                </div>
                <div className="mt-2 h-2 rounded-full bg-secondary">
                  <div
                    className="h-full rounded-full bg-foreground"
                    style={{ width: `${item.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Decision Timeline */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold">Decision Timeline</h2>
        <p className="text-sm text-muted-foreground">Recent decisions and their outcomes</p>

        <div className="mt-6 space-y-6">
          {decisionTimeline.map((day) => (
            <div key={day.date}>
              <h3 className="text-sm font-medium text-muted-foreground">{day.date}</h3>
              <div className="mt-2 space-y-3">
                {day.items.map((item, i) => (
                  <div key={i} className="flex items-start gap-3">
                    {item.outcome === 'success' ? (
                      <CheckCircle className="mt-0.5 h-4 w-4 text-success" />
                    ) : (
                      <XCircle className="mt-0.5 h-4 w-4 text-destructive" />
                    )}
                    <div className="flex-1">
                      <p className="font-medium">{item.title}</p>
                      <div className="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground">
                        <span className="rounded bg-secondary px-1.5 py-0.5">
                          {item.type}
                        </span>
                        <span>{item.agent}</span>
                      </div>
                    </div>
                    <span className="text-xs text-muted-foreground">{item.time}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
