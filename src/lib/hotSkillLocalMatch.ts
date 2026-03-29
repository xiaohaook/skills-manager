import type { HotSkill } from '../hotTypes'
import type { Skill } from '../types/domain'

/** 与 `install.rs` 里从 URL 推断文件夹名一致：去掉 .git / 查询串后取最后一段 */
function urlRepoBasename(url: string): string {
  const cleaned = url
    .trim()
    .replace(/\.git$/i, '')
    .split(/[?#]/)[0]
    .replace(/\/+$/, '')
  const parts = cleaned.split('/').filter(Boolean)
  return (parts[parts.length - 1] ?? '').toLowerCase()
}

function candidateKeys(h: HotSkill): string[] {
  const out: string[] = []
  const id = h.id?.trim().toLowerCase()
  if (id) out.push(id)
  const name = h.name?.trim().toLowerCase()
  if (name && name !== id) out.push(name)
  const u = (h.github_url ?? '').trim()
  if (u) {
    const base = urlRepoBasename(u)
    if (base && !out.includes(base)) out.push(base)
  }
  return out
}

/** 本地已扫描技能目录名索引（小写） */
export function buildLocalHotInstallIndex(skills: Skill[]): Set<string> {
  const set = new Set<string>()
  for (const s of skills) {
    const n = s.name.trim().toLowerCase()
    if (n) set.add(n)
    const leaf = s.path.split(/[/\\]/).filter(Boolean).pop()?.toLowerCase()
    if (leaf && leaf !== n) set.add(leaf)
  }
  return set
}

/** 是否与当前机器上任意来源下的技能文件夹名匹配 */
export function isHotSkillInstalledLocally(hot: HotSkill, index: Set<string>): boolean {
  return candidateKeys(hot).some(k => index.has(k))
}
