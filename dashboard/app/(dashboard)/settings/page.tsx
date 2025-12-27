'use client'

import { useState } from 'react'
import { Key, User, Copy, Check, RefreshCw } from 'lucide-react'

export default function SettingsPage() {
  const [copied, setCopied] = useState(false)
  const [apiKey] = useState('am_xxxxxxxxxxxxxxxxxxxxxxxxxxxx')
  const [email] = useState('developer@example.com')
  const [name, setName] = useState('Developer')

  const copyApiKey = () => {
    navigator.clipboard.writeText(apiKey)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-muted-foreground">
          Manage your account and API access
        </p>
      </div>

      {/* Profile */}
      <div className="rounded-lg border border-border bg-card p-6">
        <div className="flex items-center gap-3">
          <User className="h-5 w-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Profile</h2>
        </div>
        <div className="mt-6 space-y-4">
          <div>
            <label className="text-sm text-muted-foreground">Email</label>
            <input
              type="email"
              value={email}
              disabled
              className="mt-1 w-full rounded-md border border-border bg-secondary px-3 py-2 text-sm text-muted-foreground"
            />
          </div>
          <div>
            <label className="text-sm text-muted-foreground">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="mt-1 w-full rounded-md border border-border bg-background px-3 py-2 text-sm focus:border-foreground focus:outline-none"
            />
          </div>
          <button className="rounded-md bg-foreground px-4 py-2 text-sm font-medium text-background hover:bg-foreground/90">
            Save Changes
          </button>
        </div>
      </div>

      {/* API Key */}
      <div className="rounded-lg border border-border bg-card p-6">
        <div className="flex items-center gap-3">
          <Key className="h-5 w-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">API Key</h2>
        </div>
        <p className="mt-2 text-sm text-muted-foreground">
          Use this key to authenticate CLI requests
        </p>
        <div className="mt-4 flex items-center gap-2">
          <code className="flex-1 rounded-md border border-border bg-secondary px-3 py-2 font-mono text-sm">
            {apiKey}
          </code>
          <button
            onClick={copyApiKey}
            className="rounded-md border border-border p-2 hover:bg-secondary"
          >
            {copied ? (
              <Check className="h-4 w-4 text-success" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </button>
        </div>
        <div className="mt-4 flex items-center gap-4">
          <button className="flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-secondary">
            <RefreshCw className="h-4 w-4" />
            Regenerate Key
          </button>
          <span className="text-xs text-muted-foreground">
            Warning: This will invalidate your current key
          </span>
        </div>
      </div>

      {/* CLI Setup */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold">CLI Setup</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Configure your local CLI to connect to this dashboard
        </p>
        <div className="mt-4 space-y-4">
          <div>
            <p className="text-sm font-medium">Option 1: Login with API key</p>
            <code className="mt-2 block rounded-md border border-border bg-secondary px-3 py-2 font-mono text-sm">
              am auth login --api-key {apiKey}
            </code>
          </div>
          <div>
            <p className="text-sm font-medium">Option 2: Set environment variable</p>
            <code className="mt-2 block rounded-md border border-border bg-secondary px-3 py-2 font-mono text-sm">
              export AGENTMEM_API_KEY={apiKey}
            </code>
          </div>
          <div>
            <p className="text-sm font-medium">Option 3: Add to credentials file</p>
            <code className="mt-2 block rounded-md border border-border bg-secondary px-3 py-2 font-mono text-sm">
              echo &quot;AGENTMEM_API_KEY={apiKey}&quot; {'>>'} ~/.agentmem/credentials
            </code>
          </div>
        </div>
      </div>

      {/* Danger Zone */}
      <div className="rounded-lg border border-destructive/50 bg-card p-6">
        <h2 className="text-lg font-semibold text-destructive">Danger Zone</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Irreversible actions
        </p>
        <div className="mt-4 flex items-center gap-4">
          <button className="rounded-md border border-destructive px-4 py-2 text-sm text-destructive hover:bg-destructive/10">
            Delete All Data
          </button>
          <button className="rounded-md border border-destructive px-4 py-2 text-sm text-destructive hover:bg-destructive/10">
            Delete Account
          </button>
        </div>
      </div>
    </div>
  )
}
