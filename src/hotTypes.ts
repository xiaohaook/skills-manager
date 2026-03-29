export interface HotSkill {
  id: string
  name: string
  description: string
  emoji: string
  author: string
  stars: number
  installs: number
  tags: string[]
  github_url: string
  platform?: string
  /** 后端标记：克隆可能很慢 */
  largeClone?: boolean
}

export type HotPlatformId =
  | 'all'
  | 'github'
  | 'clawhub'
  | 'cursor'
  | 'gitlab'
  | 'codeberg'
  | 'huggingface'

export function resolveHotSkillPlatform(s: HotSkill): Exclude<HotPlatformId, 'all'> {
  const p = (s.platform ?? '').trim().toLowerCase()
  const u = (s.github_url ?? '').toLowerCase()
  if (p === 'clawhub') return 'clawhub'
  if (p === 'cursor') return 'cursor'
  if (p === 'gitlab' || u.includes('gitlab.com')) return 'gitlab'
  if (p === 'codeberg' || u.includes('codeberg.org')) return 'codeberg'
  if (p === 'huggingface' || p === 'hf' || u.includes('huggingface.co')) return 'huggingface'
  return 'github'
}

export const HOT_PLATFORM_SIDEBAR: Array<{ id: HotPlatformId; name: string; emoji: string }> = [
  { id: 'all', name: '全部', emoji: '🌍' },
  { id: 'github', name: 'GitHub', emoji: '🐙' },
  { id: 'clawhub', name: 'ClawHub', emoji: '🦞' },
  { id: 'cursor', name: 'Cursor', emoji: '⎈' },
  { id: 'gitlab', name: 'GitLab', emoji: '🦊' },
  { id: 'codeberg', name: 'Codeberg', emoji: '🔷' },
  { id: 'huggingface', name: 'Hugging Face', emoji: '🤗' },
]

const HOT_TAG_LABELS: Record<string, { name: string; emoji: string }> = {
  AI: { name: 'AI / LLM', emoji: '🤖' },
  Python: { name: 'Python', emoji: '🐍' },
  JavaScript: { name: 'JavaScript', emoji: '📜' },
  Frontend: { name: '前端开发', emoji: '🎨' },
  Backend: { name: '后端开发', emoji: '🔌' },
  DevOps: { name: 'DevOps', emoji: '🐳' },
  Data: { name: '数据分析', emoji: '📊' },
  Security: { name: '安全', emoji: '🔒' },
  Test: { name: '测试', emoji: '✅' },
  Media: { name: '媒体处理', emoji: '🎬' },
  Productivity: { name: '效率工具', emoji: '⚡' },
  MCP: { name: 'MCP', emoji: '🔧' },
  Learning: { name: '学习', emoji: '📚' },
  Rust: { name: 'Rust', emoji: '🦀' },
  Go: { name: 'Go', emoji: '🐹' },
  API: { name: 'API', emoji: '🔌' },
  CLI: { name: 'CLI', emoji: '💻' },
  Image: { name: '图像', emoji: '🖼️' },
}

export function hotTagDisplay(tag: string): { name: string; emoji: string } {
  return HOT_TAG_LABELS[tag] ?? { name: tag, emoji: '🏷️' }
}

export const HOT_SIDEBAR_MAX_TAGS = 7

export function buildDynamicHotCategories(
  filteredSkills: HotSkill[],
  maxTags = HOT_SIDEBAR_MAX_TAGS
): Array<{ id: string; name: string; emoji: string; count: number }> {
  const tagCounts = new Map<string, number>()
  for (const s of filteredSkills) {
    for (const t of s.tags ?? []) {
      const key = t.trim()
      if (!key) continue
      tagCounts.set(key, (tagCounts.get(key) ?? 0) + 1)
    }
  }
  const rows = [...tagCounts.entries()]
    .filter(([, n]) => n > 0)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], 'en'))
    .slice(0, maxTags)
    .map(([id, count]) => {
      const d = hotTagDisplay(id)
      return { id, name: d.name, emoji: d.emoji, count }
    })
  const untagged = filteredSkills.filter(s => !(s.tags && s.tags.length)).length
  if (untagged > 0) {
    rows.push({ id: '__other__', name: '未打标签', emoji: '📌', count: untagged })
  }
  return rows
}

export const PLATFORM_BADGE_STYLE: Record<
  Exclude<HotPlatformId, 'all'>,
  { src: string; label: string; box: string; text: string }
> = {
  github: {
    src: '/platforms/github.svg',
    label: 'GitHub',
    box: 'bg-zinc-100 border-zinc-300 dark:bg-zinc-900/60 dark:border-zinc-600/50',
    text: 'text-zinc-600 dark:text-zinc-400',
  },
  clawhub: {
    src: '/platforms/clawhub.svg',
    label: 'ClawHub',
    box: 'bg-orange-100/90 border-orange-300 dark:bg-orange-950/25 dark:border-orange-700/35',
    text: 'text-orange-900 dark:text-orange-200/80',
  },
  cursor: {
    src: '/platforms/cursor.svg',
    label: 'Cursor',
    box: 'bg-slate-100 border-slate-300 dark:bg-slate-900/60 dark:border-slate-500/35',
    text: 'text-slate-700 dark:text-slate-300',
  },
  gitlab: {
    src: '/platforms/gitlab.svg',
    label: 'GitLab',
    box: 'bg-orange-50 border-orange-300 dark:bg-orange-950/20 dark:border-orange-600/40',
    text: 'text-orange-900 dark:text-orange-200/75',
  },
  codeberg: {
    src: '/platforms/codeberg.svg',
    label: 'Codeberg',
    box: 'bg-sky-50 border-sky-300 dark:bg-sky-950/25 dark:border-sky-600/35',
    text: 'text-sky-900 dark:text-sky-200/80',
  },
  huggingface: {
    src: '/platforms/huggingface.svg',
    label: 'Hugging Face',
    box: 'bg-amber-50 border-amber-300 dark:bg-amber-950/30 dark:border-amber-600/35',
    text: 'text-amber-950 dark:text-amber-100/85',
  },
}
