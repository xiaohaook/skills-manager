import { Fragment } from 'react'
import { Plus, Trash2 } from 'lucide-react'
import type { HotPlatformId } from '../hotTypes'
import type { HotSkill } from '../hotTypes'
import type { SourceConfig } from '../types/domain'
import {
  HOT_PLATFORM_SIDEBAR,
  buildDynamicHotCategories,
  resolveHotSkillPlatform,
} from '../hotTypes'
import { getSourceColor } from '../lib/sourceColors'

type Tab = 'skills' | 'hot' | 'sources'

type ClawListItem = { id: string; name: string; displayName: string; count: number }

export function AppSidebar({
  widthPx,
  onResizeMouseDown,
  activeTab,
  onTabChange,
  clawList,
  selectedClaw,
  onSelectClaw,
  hotSkills,
  hotPlatform,
  onHotPlatform,
  hotCategory,
  onHotCategory,
  customSources,
  onRemoveCustomSource,
  onShowAddSource,
}: {
  widthPx: number
  onResizeMouseDown: (e: React.MouseEvent) => void
  activeTab: Tab
  onTabChange: (t: Tab) => void
  clawList: ClawListItem[]
  selectedClaw: string
  onSelectClaw: (name: string) => void
  hotSkills: HotSkill[]
  hotPlatform: HotPlatformId
  onHotPlatform: (p: HotPlatformId) => void
  hotCategory: string
  onHotCategory: (c: string) => void
  customSources: SourceConfig[]
  onRemoveCustomSource: (name: string) => void
  onShowAddSource: () => void
}) {
  return (
    <aside
      className="relative flex shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/95 p-3 pr-1 backdrop-blur-md dark:border-zinc-800 dark:bg-zinc-950/95"
      style={{ width: widthPx }}
    >
      <div className="mb-3 flex shrink-0 items-center gap-2 rounded-lg px-0.5 py-1">
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-emerald-500 text-xl shadow-sm">
          🧩
        </div>
        <h1 className="text-lg font-semibold tracking-tight text-zinc-800 dark:text-zinc-100">
          Skills Manager
        </h1>
      </div>

      <nav
        data-no-drag
        className="min-h-0 flex-1 space-y-1 overflow-y-auto overflow-x-hidden pb-2 pr-1"
        aria-label="主导航"
      >
        <div className="mb-3 flex gap-1" role="tablist" aria-label="主标签">
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === 'skills'}
            onClick={() => onTabChange('skills')}
            className={`flex-1 rounded-lg py-2 text-sm font-medium ${
              activeTab === 'skills'
                ? 'bg-zinc-300 text-zinc-900 shadow-sm dark:bg-zinc-700 dark:text-white'
                : 'text-zinc-600 hover:bg-zinc-200 dark:text-zinc-400 dark:hover:bg-zinc-800'
            }`}
          >
            我的技能
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === 'hot'}
            onClick={() => onTabChange('hot')}
            className={`flex-1 rounded-lg py-2 text-sm font-medium ${
              activeTab === 'hot'
                ? 'bg-zinc-300 text-zinc-900 shadow-sm dark:bg-zinc-700 dark:text-white'
                : 'text-zinc-600 hover:bg-zinc-200 dark:text-zinc-400 dark:hover:bg-zinc-800'
            }`}
          >
            热榜
          </button>
        </div>

        {activeTab === 'skills' && (
          <div className="space-y-1">
            <div className="mb-2 text-sm text-zinc-500 dark:text-zinc-400">来源</div>
            {clawList.map((group, index) => {
              const dotColor = getSourceColor(group.displayName)
              return (
                <Fragment key={group.id}>
                  <button
                    type="button"
                    aria-current={selectedClaw === group.name ? 'true' : undefined}
                    onClick={() => onSelectClaw(group.name)}
                    className={`flex w-full items-center justify-between rounded-lg px-3 py-2 ${
                      selectedClaw === group.name
                        ? 'bg-zinc-200 text-zinc-900 dark:bg-zinc-800 dark:text-white'
                        : 'text-zinc-500 hover:bg-zinc-200/80 dark:text-zinc-400 dark:hover:bg-zinc-800/50'
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <div className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: dotColor }} />
                      <span className="truncate">{group.displayName}</span>
                    </div>
                    <span className="rounded-full bg-zinc-300 px-2 py-0.5 text-xs dark:bg-zinc-700">
                      {group.count}
                    </span>
                  </button>
                  {index === 0 && <div className="mx-3 my-1 h-px bg-zinc-200 dark:bg-zinc-800" />}
                </Fragment>
              )
            })}
          </div>
        )}

        {activeTab === 'hot' && (
          <div className="space-y-3">
            <div className="mb-2 text-sm text-zinc-500 dark:text-zinc-400">🌐 平台</div>
            {HOT_PLATFORM_SIDEBAR.map(plat => {
              const count =
                plat.id === 'all'
                  ? hotSkills.length
                  : hotSkills.filter(s => resolveHotSkillPlatform(s) === plat.id).length
              return (
                <button
                  key={plat.id}
                  type="button"
                  aria-current={hotPlatform === plat.id ? 'true' : undefined}
                  onClick={() => onHotPlatform(plat.id)}
                  className={`flex w-full items-center justify-between rounded-lg px-3 py-2 ${
                    hotPlatform === plat.id
                      ? 'bg-zinc-200 text-zinc-900 dark:bg-zinc-800 dark:text-white'
                      : 'text-zinc-500 hover:bg-zinc-200/80 dark:text-zinc-400 dark:hover:bg-zinc-800/50'
                  }`}
                >
                  <div className="flex min-w-0 items-center gap-2">
                    <span className="shrink-0 text-lg">{plat.emoji}</span>
                    <span className="truncate text-left">{plat.name}</span>
                  </div>
                  <span className="shrink-0 rounded-full bg-zinc-300 px-2 py-0.5 text-xs dark:bg-zinc-700">
                    {count}
                  </span>
                </button>
              )
            })}

            <div className="my-2 border-t border-zinc-200 dark:border-zinc-800" />

            <div className="mb-2 text-sm text-zinc-500 dark:text-zinc-400">🔥 热门分类</div>
            {(() => {
              const filteredSkills =
                hotPlatform === 'all'
                  ? hotSkills
                  : hotSkills.filter(s => resolveHotSkillPlatform(s) === hotPlatform)
              const dynamicRows = buildDynamicHotCategories(filteredSkills)
              type CatRow = { id: string; name: string; emoji: string; count: number }
              const categories: CatRow[] = [
                { id: 'all', name: '全部', emoji: '📦', count: filteredSkills.length },
                ...dynamicRows,
              ]

              return categories.map(cat => (
                <button
                  key={cat.id}
                  type="button"
                  aria-current={hotCategory === cat.id ? 'true' : undefined}
                  onClick={() => onHotCategory(cat.id)}
                  className={`flex w-full items-center justify-between rounded-lg px-3 py-2 ${
                    hotCategory === cat.id
                      ? 'bg-zinc-200 text-zinc-900 dark:bg-zinc-800 dark:text-white'
                      : 'text-zinc-500 hover:bg-zinc-200/80 dark:text-zinc-400 dark:hover:bg-zinc-800/50'
                  }`}
                >
                  <div className="flex min-w-0 items-center gap-2">
                    <span className="shrink-0 text-lg">{cat.emoji}</span>
                    <span className="truncate text-left">{cat.name}</span>
                  </div>
                  <span className="shrink-0 rounded-full bg-zinc-300 px-2 py-0.5 text-xs dark:bg-zinc-700">
                    {cat.count}
                  </span>
                </button>
              ))
            })()}
          </div>
        )}

        {activeTab === 'sources' && (
          <div className="space-y-2">
            <div className="mb-2 text-sm text-zinc-500 dark:text-zinc-400">自定义来源</div>
            {customSources.map(source => (
              <div
                key={source.name}
                className="flex items-center justify-between rounded-lg bg-zinc-100 p-2 dark:bg-zinc-800"
              >
                <div className="truncate text-sm">{source.name}</div>
                <button
                  type="button"
                  aria-label={`移除来源 ${source.name}`}
                  onClick={() => onRemoveCustomSource(source.name)}
                  className="text-red-400 hover:text-red-300"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            ))}
            <button
              type="button"
              onClick={onShowAddSource}
              className="w-full rounded-lg border border-dashed border-zinc-400 py-2 text-zinc-500 hover:border-zinc-500 hover:text-zinc-700 dark:border-zinc-600 dark:text-zinc-400 dark:hover:border-zinc-500 dark:hover:text-zinc-300"
            >
              <Plus className="mr-1 inline h-4 w-4" /> 添加来源
            </button>
          </div>
        )}
      </nav>
      <div
        data-no-drag
        role="separator"
        title="拖拽调整宽度"
        aria-label="调整侧栏宽度"
        className="absolute bottom-0 right-0 top-0 z-20 w-1.5 cursor-col-resize hover:bg-emerald-500/35 active:bg-emerald-500/55"
        onMouseDown={onResizeMouseDown}
      />
    </aside>
  )
}
