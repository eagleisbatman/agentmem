import {
  FolderKanban,
  Brain,
  Clock,
  Zap,
  ArrowUpRight,
  CheckCircle,
  XCircle,
  TrendingUp,
} from 'lucide-react'

// Placeholder data - in production, this would fetch from API
const stats = [
  { name: 'Projects', value: '3', icon: FolderKanban, change: '+1 this week' },
  { name: 'Memories', value: '47', icon: Brain, change: '+12 this week' },
  { name: 'Sessions', value: '23', icon: Clock, change: '8 today' },
  { name: 'Tokens Used', value: '847k', icon: Zap, change: '~$4.23' },
]

const recentActivity = [
  {
    id: 1,
    type: 'decision',
    title: 'Use MDC format for Cursor',
    project: 'agentmem',
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'success',
    time: '14:32',
  },
  {
    id: 2,
    type: 'decision',
    title: 'Wrapper script for Codex',
    project: 'agentmem',
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'success',
    time: '14:28',
  },
  {
    id: 3,
    type: 'correction',
    title: 'Fixed Gemini settings format',
    project: 'agentmem',
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'failed',
    time: '11:45',
  },
  {
    id: 4,
    type: 'decision',
    title: 'Multi-agent hook system',
    project: 'agentmem',
    agent: 'Claude Code',
    model: 'claude-sonnet-4-20250514',
    outcome: 'success',
    time: '10:30',
  },
]

const projects = [
  { name: 'agentmem', memories: 47, lastActive: 'Active now', isActive: true },
  { name: 'my-saas-app', memories: 23, lastActive: '2 hrs ago', isActive: false },
  { name: 'mobile-app', memories: 12, lastActive: 'Yesterday', isActive: false },
]

export default function DashboardPage() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Overview of your AI-assisted development
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <div
            key={stat.name}
            className="rounded-lg border border-border bg-card p-6"
          >
            <div className="flex items-center justify-between">
              <stat.icon className="h-5 w-5 text-muted-foreground" />
              <span className="text-xs text-muted-foreground">{stat.change}</span>
            </div>
            <div className="mt-4">
              <p className="text-3xl font-bold">{stat.value}</p>
              <p className="text-sm text-muted-foreground">{stat.name}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="grid gap-8 lg:grid-cols-3">
        {/* Projects */}
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Projects</h2>
            <a
              href="/projects"
              className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
            >
              View all
              <ArrowUpRight className="h-3 w-3" />
            </a>
          </div>
          <div className="mt-4 space-y-4">
            {projects.map((project) => (
              <div
                key={project.name}
                className="flex items-center justify-between"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`h-2 w-2 rounded-full ${
                      project.isActive ? 'bg-success' : 'bg-muted'
                    }`}
                  />
                  <div>
                    <p className="font-medium">{project.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {project.memories} memories
                    </p>
                  </div>
                </div>
                <span className="text-xs text-muted-foreground">
                  {project.lastActive}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Recent Activity */}
        <div className="col-span-2 rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Recent Activity</h2>
            <a
              href="/analytics"
              className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
            >
              View timeline
              <ArrowUpRight className="h-3 w-3" />
            </a>
          </div>
          <div className="mt-4 space-y-4">
            {recentActivity.map((activity) => (
              <div
                key={activity.id}
                className="flex items-start justify-between border-b border-border pb-4 last:border-0 last:pb-0"
              >
                <div className="flex items-start gap-3">
                  {activity.outcome === 'success' ? (
                    <CheckCircle className="mt-0.5 h-4 w-4 text-success" />
                  ) : (
                    <XCircle className="mt-0.5 h-4 w-4 text-destructive" />
                  )}
                  <div>
                    <p className="font-medium">{activity.title}</p>
                    <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                      <span className="rounded bg-secondary px-1.5 py-0.5">
                        {activity.type}
                      </span>
                      <span>{activity.project}</span>
                      <span>via {activity.agent}</span>
                    </div>
                  </div>
                </div>
                <span className="text-xs text-muted-foreground">
                  {activity.time}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Insights */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold">Insights</h2>
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="flex items-center gap-3">
            <TrendingUp className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium">This week</p>
              <p className="text-xs text-muted-foreground">
                23 decisions, 7 corrections (77% success)
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Zap className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium">Peak usage</p>
              <p className="text-xs text-muted-foreground">
                Wednesday 2pm (refactoring session)
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Brain className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium">Top model</p>
              <p className="text-xs text-muted-foreground">
                Claude Sonnet (67% of tokens)
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <CheckCircle className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium">Success rate</p>
              <p className="text-xs text-muted-foreground">
                77% decisions successful
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
