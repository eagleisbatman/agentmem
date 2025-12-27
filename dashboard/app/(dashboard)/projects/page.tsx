import { FolderKanban, Clock, Brain, Zap } from 'lucide-react'

// Placeholder data
const projects = [
  {
    id: '1',
    name: 'agentmem',
    path: '/Users/eagle/agentmem',
    memories: 47,
    sessions: 23,
    tokensUsed: 568000,
    lastActive: 'Active now',
    isActive: true,
    agents: ['Claude Code', 'Cursor'],
  },
  {
    id: '2',
    name: 'my-saas-app',
    path: '/Users/eagle/my-saas-app',
    memories: 23,
    sessions: 15,
    tokensUsed: 237000,
    lastActive: '2 hours ago',
    isActive: false,
    agents: ['Cursor', 'Gemini CLI'],
  },
  {
    id: '3',
    name: 'mobile-app',
    path: '/Users/eagle/mobile-app',
    memories: 12,
    sessions: 8,
    tokensUsed: 42000,
    lastActive: 'Yesterday',
    isActive: false,
    agents: ['Claude Code'],
  },
]

function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(1)}M`
  }
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(0)}k`
  }
  return tokens.toString()
}

export default function ProjectsPage() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Projects</h1>
        <p className="text-muted-foreground">
          All registered projects across your machines
        </p>
      </div>

      {/* Projects Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {projects.map((project) => (
          <a
            key={project.id}
            href={`/projects/${project.id}`}
            className="group rounded-lg border border-border bg-card p-6 transition-colors hover:border-foreground/20"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <FolderKanban className="h-5 w-5 text-muted-foreground" />
                <div>
                  <h3 className="font-semibold group-hover:text-foreground">
                    {project.name}
                  </h3>
                  <p className="text-xs text-muted-foreground">{project.path}</p>
                </div>
              </div>
              <div
                className={`h-2 w-2 rounded-full ${
                  project.isActive ? 'bg-success' : 'bg-muted'
                }`}
              />
            </div>

            <div className="mt-6 grid grid-cols-3 gap-4">
              <div>
                <div className="flex items-center gap-1.5 text-muted-foreground">
                  <Brain className="h-3.5 w-3.5" />
                  <span className="text-xs">Memories</span>
                </div>
                <p className="mt-1 text-lg font-semibold">{project.memories}</p>
              </div>
              <div>
                <div className="flex items-center gap-1.5 text-muted-foreground">
                  <Clock className="h-3.5 w-3.5" />
                  <span className="text-xs">Sessions</span>
                </div>
                <p className="mt-1 text-lg font-semibold">{project.sessions}</p>
              </div>
              <div>
                <div className="flex items-center gap-1.5 text-muted-foreground">
                  <Zap className="h-3.5 w-3.5" />
                  <span className="text-xs">Tokens</span>
                </div>
                <p className="mt-1 text-lg font-semibold">
                  {formatTokens(project.tokensUsed)}
                </p>
              </div>
            </div>

            <div className="mt-4 flex items-center justify-between border-t border-border pt-4">
              <div className="flex gap-1">
                {project.agents.map((agent) => (
                  <span
                    key={agent}
                    className="rounded bg-secondary px-2 py-0.5 text-xs"
                  >
                    {agent}
                  </span>
                ))}
              </div>
              <span className="text-xs text-muted-foreground">
                {project.lastActive}
              </span>
            </div>
          </a>
        ))}
      </div>
    </div>
  )
}
