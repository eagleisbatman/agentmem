'use client'

import { useState } from 'react'
import {
  Brain,
  Globe,
  FolderKanban,
  CheckCircle,
  XCircle,
  AlertCircle,
  ArrowUpRight,
  Search,
} from 'lucide-react'

// Placeholder data
const memories = [
  {
    id: '1',
    title: 'Use MDC format for Cursor rules',
    content: 'Cursor IDE uses .cursor/rules/*.mdc format for rules, not the legacy .cursorrules',
    memoryType: 'decision',
    scope: 'global',
    project: null,
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'success',
    createdAt: '2024-01-15T14:32:00Z',
  },
  {
    id: '2',
    title: 'Use PostgreSQL for the main database',
    content: 'PostgreSQL chosen over MongoDB for relational data integrity',
    memoryType: 'decision',
    scope: 'project',
    project: 'agentmem',
    agent: 'Cursor',
    model: 'gpt-4o',
    outcome: 'success',
    createdAt: '2024-01-15T10:30:00Z',
  },
  {
    id: '3',
    title: 'Gemini CLI settings.json expects object for hooks',
    content: 'Gemini CLI hooks config expects an object, not an array. Setting hooks to array breaks CLI.',
    memoryType: 'correction',
    scope: 'global',
    project: null,
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'failed',
    createdAt: '2024-01-15T11:45:00Z',
  },
  {
    id: '4',
    title: 'Always use pnpm over npm',
    content: 'User prefers pnpm for package management across all projects',
    memoryType: 'pattern',
    scope: 'global',
    project: null,
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'success',
    createdAt: '2024-01-14T16:00:00Z',
  },
  {
    id: '5',
    title: 'API endpoint at localhost:3000',
    content: 'Development API server runs at http://localhost:3000',
    memoryType: 'infrastructure',
    scope: 'project',
    project: 'my-saas-app',
    agent: 'Gemini CLI',
    model: 'gemini-2.0-flash',
    outcome: 'unknown',
    createdAt: '2024-01-14T09:15:00Z',
  },
]

const memoryTypes = [
  { value: 'all', label: 'All Types' },
  { value: 'decision', label: 'Decisions' },
  { value: 'correction', label: 'Corrections' },
  { value: 'pattern', label: 'Patterns' },
  { value: 'gotcha', label: 'Gotchas' },
  { value: 'infrastructure', label: 'Infrastructure' },
]

const scopes = [
  { value: 'all', label: 'All Scopes' },
  { value: 'global', label: 'Global' },
  { value: 'project', label: 'Project' },
]

function getOutcomeIcon(outcome: string) {
  switch (outcome) {
    case 'success':
      return <CheckCircle className="h-4 w-4 text-success" />
    case 'failed':
      return <XCircle className="h-4 w-4 text-destructive" />
    default:
      return <AlertCircle className="h-4 w-4 text-muted-foreground" />
  }
}

function getScopeIcon(scope: string) {
  return scope === 'global' ? (
    <Globe className="h-3.5 w-3.5" />
  ) : (
    <FolderKanban className="h-3.5 w-3.5" />
  )
}

export default function MemoriesPage() {
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState('all')
  const [scopeFilter, setScopeFilter] = useState('all')

  const filteredMemories = memories.filter((memory) => {
    const matchesSearch =
      memory.title.toLowerCase().includes(search.toLowerCase()) ||
      memory.content?.toLowerCase().includes(search.toLowerCase())
    const matchesType = typeFilter === 'all' || memory.memoryType === typeFilter
    const matchesScope = scopeFilter === 'all' || memory.scope === scopeFilter
    return matchesSearch && matchesType && matchesScope
  })

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Memories</h1>
        <p className="text-muted-foreground">
          All memories across your projects
        </p>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="relative flex-1 min-w-[200px] max-w-md">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search memories..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full rounded-md border border-border bg-background py-2 pl-10 pr-4 text-sm focus:border-foreground focus:outline-none"
          />
        </div>
        <select
          value={typeFilter}
          onChange={(e) => setTypeFilter(e.target.value)}
          className="rounded-md border border-border bg-background px-3 py-2 text-sm focus:border-foreground focus:outline-none"
        >
          {memoryTypes.map((type) => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
        <select
          value={scopeFilter}
          onChange={(e) => setScopeFilter(e.target.value)}
          className="rounded-md border border-border bg-background px-3 py-2 text-sm focus:border-foreground focus:outline-none"
        >
          {scopes.map((scope) => (
            <option key={scope.value} value={scope.value}>
              {scope.label}
            </option>
          ))}
        </select>
      </div>

      {/* Memories List */}
      <div className="space-y-4">
        {filteredMemories.map((memory) => (
          <div
            key={memory.id}
            className="rounded-lg border border-border bg-card p-6"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start gap-3">
                {getOutcomeIcon(memory.outcome)}
                <div>
                  <h3 className="font-semibold">{memory.title}</h3>
                  {memory.content && (
                    <p className="mt-1 text-sm text-muted-foreground">
                      {memory.content}
                    </p>
                  )}
                </div>
              </div>
              {memory.scope === 'project' && (
                <button className="flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground hover:bg-secondary hover:text-foreground">
                  Promote
                  <ArrowUpRight className="h-3 w-3" />
                </button>
              )}
            </div>

            <div className="mt-4 flex flex-wrap items-center gap-3 text-xs">
              <span className="flex items-center gap-1.5 rounded bg-secondary px-2 py-1">
                <Brain className="h-3 w-3" />
                {memory.memoryType}
              </span>
              <span className="flex items-center gap-1.5 rounded bg-secondary px-2 py-1">
                {getScopeIcon(memory.scope)}
                {memory.scope}
                {memory.project && `: ${memory.project}`}
              </span>
              <span className="text-muted-foreground">
                via {memory.agent} ({memory.model})
              </span>
              <span className="text-muted-foreground">
                {new Date(memory.createdAt).toLocaleDateString()}
              </span>
            </div>
          </div>
        ))}

        {filteredMemories.length === 0 && (
          <div className="rounded-lg border border-dashed border-border p-12 text-center">
            <Brain className="mx-auto h-12 w-12 text-muted-foreground" />
            <h3 className="mt-4 font-semibold">No memories found</h3>
            <p className="mt-1 text-sm text-muted-foreground">
              Try adjusting your search or filters
            </p>
          </div>
        )}
      </div>
    </div>
  )
}
