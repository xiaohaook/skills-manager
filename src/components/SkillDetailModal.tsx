import { useEffect, useRef } from 'react'
import { CheckCircle, Terminal, XCircle } from 'lucide-react'
import type { Skill } from '../types/domain'

export function SkillDetailModal({
  skill,
  onClose,
  onBrewFix,
  brewBusy,
}: {
  skill: Skill
  onClose: () => void
  /** 对本技能 `missingBins` 发起 Homebrew 安装确认 */
  onBrewFix?: () => void
  brewBusy?: boolean
}) {
  const panelRef = useRef<HTMLDivElement>(null)
  const missingSet = new Set(skill.missingBins ?? [])

  useEffect(() => {
    const el = panelRef.current
    if (!el) return
    const selectors =
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    const nodes = el.querySelectorAll<HTMLElement>(selectors)
    const list = Array.from(nodes).filter(n => !n.hasAttribute('disabled'))
    if (list.length === 0) return
    const first = list[0]
    const last = list[list.length - 1]
    first.focus()

    function onKey(e: KeyboardEvent) {
      if (e.key !== 'Tab') return
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault()
          last.focus()
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault()
          first.focus()
        }
      }
    }
    el.addEventListener('keydown', onKey)
    return () => el.removeEventListener('keydown', onKey)
  }, [skill])

  return (
    <div
      role="presentation"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4 backdrop-blur-sm dark:bg-black/70"
      onClick={onClose}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="skill-detail-title"
        className="max-h-[90vh] w-full max-w-2xl overflow-auto rounded-xl border border-zinc-200 bg-white dark:border-zinc-700 dark:bg-zinc-800"
        onClick={e => e.stopPropagation()}
      >
        <div className="p-6">
          <div className="mb-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <span className="text-3xl">{skill.emoji}</span>
              <h2 id="skill-detail-title" className="text-2xl font-bold">
                {skill.name}
              </h2>
            </div>
            <button
              type="button"
              aria-label="关闭"
              onClick={onClose}
              className="text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-white"
            >
              ✕
            </button>
          </div>
          <div className="mb-4">
            <h3 className="mb-1 text-sm text-zinc-500 dark:text-zinc-400">描述</h3>
            <p className="text-zinc-900 dark:text-zinc-100">{skill.description}</p>
          </div>
          <div className="mb-4 grid grid-cols-2 gap-4">
            <div>
              <h3 className="mb-1 text-sm text-zinc-500 dark:text-zinc-400">来源</h3>
              <p className="text-zinc-900 dark:text-zinc-100">{skill.source}</p>
            </div>
            <div>
              <h3 className="mb-1 text-sm text-zinc-500 dark:text-zinc-400">状态</h3>
              <div className="flex items-center gap-2">
                {skill.ready ? (
                  <>
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <span>就绪</span>
                  </>
                ) : (
                  <>
                    <XCircle className="h-4 w-4 text-red-500" />
                    <span>依赖缺失</span>
                  </>
                )}
              </div>
            </div>
          </div>
          {skill.requires.length > 0 && (
            <div className="mb-4">
              <h3 className="mb-1 text-sm text-zinc-500 dark:text-zinc-400">依赖（bins）</h3>
              <p className="mb-2 text-xs text-zinc-500 dark:text-zinc-600">
                绿色：已在 PATH 中找到；橙色：未找到（可尝试用 Homebrew 安装，部分名称已做 formula 映射）。
              </p>
              <div className="flex flex-wrap gap-2">
                {skill.requires.map(bin => (
                  <span
                    key={bin}
                    className={
                      missingSet.has(bin)
                        ? 'rounded bg-orange-900/35 px-2 py-1 text-xs text-orange-300 ring-1 ring-orange-600/40'
                        : 'rounded bg-emerald-900/25 px-2 py-1 text-xs text-emerald-300'
                    }
                  >
                    {bin}
                  </span>
                ))}
              </div>
            </div>
          )}
          {!skill.ready && (skill.missingBins?.length ?? 0) > 0 && onBrewFix && (
            <div className="mb-4">
              <button
                type="button"
                disabled={brewBusy}
                onClick={onBrewFix}
                className="inline-flex items-center gap-2 rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-700 disabled:cursor-not-allowed disabled:opacity-60"
              >
                <Terminal className="h-4 w-4 shrink-0" />
                {brewBusy ? '安装进行中…' : `一键尝试 Homebrew 安装缺失项（${skill.missingBins!.length}）`}
              </button>
            </div>
          )}
          <div className="mt-6 text-xs text-zinc-500 dark:text-zinc-500">
            <p>路径：{skill.path}</p>
          </div>
        </div>
      </div>
    </div>
  )
}
