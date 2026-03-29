import { FolderOpen } from 'lucide-react'
import type { Claw, Skill } from '../types/domain'
import { getSourceColor } from '../lib/sourceColors'

export function SkillContextMenu({
  menu,
  claws,
  selectedClaw,
  clawExistsMap,
  onCopyTo,
  onShowInFinder,
  onEdit,
  onDelete,
  onDismiss,
}: {
  menu: { left: number; top: number; skill: Skill }
  claws: Claw[]
  selectedClaw: string
  clawExistsMap: Record<string, boolean>
  onCopyTo: (claw: Claw) => void
  onShowInFinder: () => void
  onEdit: () => void
  onDelete: () => void
  onDismiss: () => void
}) {
  return (
    <>
      <div
        role="menu"
        className="fixed z-50 min-w-[200px] rounded-xl border border-zinc-200 bg-white py-2 shadow-2xl dark:border-zinc-600 dark:bg-zinc-800"
        style={{ left: menu.left, top: menu.top }}
      >
        <div className="px-3 py-1 text-xs text-zinc-500 dark:text-zinc-400">复制到</div>
        {claws.map(claw => {
          if (claw.name === selectedClaw) return null
          const dotColor = getSourceColor(claw.name)
          return (
            <button
              key={claw.id}
              type="button"
              role="menuitem"
              onClick={e => {
                e.preventDefault()
                onCopyTo(claw)
              }}
              className="flex w-full items-center justify-between px-3 py-2 text-left hover:bg-zinc-100 dark:hover:bg-zinc-700"
            >
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full" style={{ backgroundColor: dotColor }} />
                <span>{claw.name}</span>
              </div>
              {clawExistsMap[claw.id] && <span className="text-xs text-green-400">已有</span>}
            </button>
          )
        })}
        <div className="my-2 border-t border-zinc-200 dark:border-zinc-600" />
        <button
          type="button"
          role="menuitem"
          onClick={e => {
            e.preventDefault()
            onShowInFinder()
          }}
          className="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-zinc-100 dark:hover:bg-zinc-700"
        >
          <FolderOpen className="h-4 w-4" /> 在 Finder 中显示
        </button>
        <button
          type="button"
          role="menuitem"
          onClick={e => {
            e.preventDefault()
            onEdit()
          }}
          className="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-zinc-100 dark:hover:bg-zinc-700"
        >
          编辑文档（SKILL / AGENTS）
        </button>
        <div className="my-1 border-t border-zinc-200 dark:border-zinc-600" />
        <button
          type="button"
          role="menuitem"
          onClick={e => {
            e.preventDefault()
            onDelete()
          }}
          className="flex w-full items-center gap-2 px-3 py-2 text-left text-red-500 hover:bg-zinc-100 dark:text-red-400 dark:hover:bg-zinc-700"
        >
          删除 Skill
        </button>
      </div>
      <div className="fixed inset-0 z-40" role="presentation" onClick={onDismiss} aria-hidden />
    </>
  )
}
