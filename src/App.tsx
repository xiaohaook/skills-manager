import { useState, useEffect, useRef, useDeferredValue, useMemo, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Search, FolderOpen, CheckCircle, XCircle, Flame, Trash2, Download, RefreshCw, Terminal } from 'lucide-react'
import { Toast } from './components/Toast'
import { HotSkillPlatformBadge } from './components/HotSkillPlatformBadge'
import { ConfirmDialog } from './components/ConfirmDialog'
import { SkillDetailModal } from './components/SkillDetailModal'
import { SkillContextMenu } from './components/SkillContextMenu'
import { AppSidebar } from './components/AppSidebar'
import type { HotSkill, HotPlatformId } from './hotTypes'
import { buildDynamicHotCategories, resolveHotSkillPlatform } from './hotTypes'
import type { Skill, Claw, SourceConfig } from './types/domain'
import { getSourceColor } from './lib/sourceColors'
import { clampMenuPosition } from './lib/clampMenuPosition'
import { buildLocalHotInstallIndex, isHotSkillInstalledLocally } from './lib/hotSkillLocalMatch'

const SIDEBAR_WIDTH_KEY = 'skills-manager-sidebar-w'
const SIDEBAR_MIN = 200
const SIDEBAR_MAX = 520
const SIDEBAR_DEFAULT = 268

function normalizeSkillsPath(p: string): string {
  const t = p.trim()
  return t.endsWith('/') ? t : `${t}/`
}

type ConfirmState =
  | null
  | { kind: 'largeClone'; hotSkill: HotSkill }
  | { kind: 'copyOverwrite'; claw: Claw; skill: Skill }
  | { kind: 'batchOverwrite'; claw: Claw; conflictNames: string[] }

function App() {
  const [skills, setSkills] = useState<Skill[]>([])
  const [claws, setClaws] = useState<Claw[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedClaw, setSelectedClaw] = useState('all')
  const [search, setSearch] = useState('')
  const deferredSearch = useDeferredValue(search)
  const searchInputRef = useRef<HTMLInputElement>(null)

  const [menu, setMenu] = useState<{ left: number; top: number; skill: Skill } | null>(null)
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null)
  const [detailSkill, setDetailSkill] = useState<Skill | null>(null)
  const [selectedSkills, setSelectedSkills] = useState<Set<string>>(new Set())
  const [clawExistsMap, setClawExistsMap] = useState<Record<string, boolean>>({})
  const [confirm, setConfirm] = useState<ConfirmState>(null)
  const [brewDialog, setBrewDialog] = useState<{ bins: string[] } | null>(null)
  const [brewInstalling, setBrewInstalling] = useState(false)

  const [activeTab, setActiveTab] = useState<'skills' | 'hot' | 'sources'>('skills')
  const [hotSkills, setHotSkills] = useState<HotSkill[]>([])
  const [hotLoading, setHotLoading] = useState(true)
  const [customSources, setCustomSources] = useState<SourceConfig[]>([])
  const [installUrl, setInstallUrl] = useState('')
  const [installTargetId, setInstallTargetId] = useState('')
  const [newSourceName, setNewSourceName] = useState('')
  const [newSourcePath, setNewSourcePath] = useState('')
  const [showAddSource, setShowAddSource] = useState(false)
  const [installing, setInstalling] = useState(false)
  const [installingSkillId, setInstallingSkillId] = useState<string | null>(null)

  const [hotCategory, setHotCategory] = useState('all')
  const [hotPlatform, setHotPlatform] = useState<HotPlatformId>('all')

  const [sidebarW, setSidebarW] = useState(() => {
    if (typeof localStorage === 'undefined') return SIDEBAR_DEFAULT
    const raw = localStorage.getItem(SIDEBAR_WIDTH_KEY)
    const n = raw ? parseInt(raw, 10) : SIDEBAR_DEFAULT
    if (Number.isNaN(n)) return SIDEBAR_DEFAULT
    return Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, n))
  })
  const sidebarDrag = useRef<{ startX: number; startW: number } | null>(null)

  const installTargetPath = useCallback(() => {
    const c = claws.find(x => x.id === installTargetId) ?? claws[0]
    return c?.skills_path ?? '~/.openclaw/skills/'
  }, [claws, installTargetId])

  useEffect(() => {
    localStorage.setItem(SIDEBAR_WIDTH_KEY, String(sidebarW))
  }, [sidebarW])

  useEffect(() => {
    if (!claws.length) return
    if (!installTargetId || !claws.some(c => c.id === installTargetId)) {
      setInstallTargetId(claws[0].id)
    }
  }, [claws, installTargetId])

  useEffect(() => {
    function onMove(e: MouseEvent) {
      const d = sidebarDrag.current
      if (!d) return
      const next = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, d.startW + (e.clientX - d.startX)))
      setSidebarW(next)
    }
    function onUp() {
      sidebarDrag.current = null
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [])

  function onSidebarResizeMouseDown(e: React.MouseEvent) {
    e.preventDefault()
    sidebarDrag.current = { startX: e.clientX, startW: sidebarW }
  }

  useEffect(() => {
    load()
    void loadHotSkills()
    loadCustomSources()
  }, [])

  useEffect(() => {
    if (!detailSkill && !menu && !confirm && !brewDialog) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        if (brewInstalling) return
        setDetailSkill(null)
        setMenu(null)
        setConfirm(null)
        setBrewDialog(null)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [detailSkill, menu, confirm, brewDialog, brewInstalling])

  useEffect(() => {
    setDetailSkill(prev => {
      if (!prev) return null
      const updated = skills.find(s => s.path === prev.path)
      return updated ?? null
    })
  }, [skills])

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (
        e.key === '/' &&
        activeTab === 'skills' &&
        document.activeElement?.tagName !== 'INPUT' &&
        document.activeElement?.tagName !== 'TEXTAREA'
      ) {
        e.preventDefault()
        searchInputRef.current?.focus()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [activeTab])

  useEffect(() => {
    setHotCategory('all')
  }, [hotPlatform])

  useEffect(() => {
    const filtered =
      hotPlatform === 'all'
        ? hotSkills
        : hotSkills.filter(s => resolveHotSkillPlatform(s) === hotPlatform)
    const sidebar = buildDynamicHotCategories(filtered)
    const allowed = new Set<string>(['all', ...sidebar.map(c => c.id)])
    if (!allowed.has(hotCategory)) setHotCategory('all')
  }, [hotSkills, hotPlatform, hotCategory])

  async function loadHotSkills() {
    setHotLoading(true)
    try {
      const data = await invoke<HotSkill[]>('get_hot_skills')
      setHotSkills(data)
    } catch (err) {
      console.error('Failed to load hot skills:', err)
      setToast({
        message: `热榜加载失败：${err}。可检查网络后点「强制刷新」或清除缓存重试。`,
        type: 'error',
      })
    } finally {
      setHotLoading(false)
    }
  }

  async function refreshHotSkillsClearCache() {
    try {
      await invoke('clear_hot_skills_cache')
    } catch {
      /* 忽略 */
    }
    await loadHotSkills()
  }

  async function openSkillContextMenu(clientX: number, clientY: number, skill: Skill) {
    const { left, top } = clampMenuPosition(clientX, clientY)
    setMenu({ left, top, skill })
    const entries = await Promise.all(
      claws.map(async claw => {
        const targetPath = `${claw.skills_path}${skill.name}`
        const exists = await invoke<boolean>('check_path_exists', { path: targetPath })
        return [claw.id, exists] as const
      })
    )
    const existsMap: Record<string, boolean> = {}
    for (const [id, ex] of entries) {
      existsMap[id] = ex
    }
    setClawExistsMap(existsMap)
  }

  async function loadCustomSources() {
    try {
      const data = await invoke<SourceConfig[]>('get_custom_sources')
      setCustomSources(data)
    } catch (err) {
      console.error('Failed to load custom sources:', err)
    }
  }

  async function load() {
    try {
      const [clawsData, customData] = await Promise.all([
        invoke<Claw[]>('get_local_claws'),
        invoke<SourceConfig[]>('get_custom_sources').catch(() => [] as SourceConfig[]),
      ])

      const extra: Claw[] = []
      for (const s of customData) {
        if (s.enabled === false) continue
        const skillsPath = normalizeSkillsPath(s.path)
        if (clawsData.some(c => c.skills_path === skillsPath)) continue
        extra.push({
          id: `custom:${s.name}`,
          name: s.name,
          skills_path: skillsPath,
          is_local: true,
        })
      }
      const mergedClaws = [...clawsData, ...extra]

      setClaws(mergedClaws)
      const skillsData = await invoke<Skill[]>('scan_skills', { claws: mergedClaws })
      const skillsWithSource = skillsData.map(skill => ({
        ...skill,
        tags: skill.requires.length > 0 ? ['CLI'] : ['No Deps'],
      }))
      setSkills(skillsWithSource)
      setLoading(false)
    } catch (err) {
      console.error('Load failed:', err)
      setToast({
        message: `扫描技能失败：${err}。请确认本机主目录可读，或稍后点「刷新」重试。`,
        type: 'error',
      })
      setLoading(false)
    }
  }

  const clawList = useMemo(
    () => [
      { id: 'all', name: 'all', displayName: '全部', count: skills.length },
      ...mergeSources(skills).map(source => {
        const sourceSkills = skills.filter(s => s.source === source.name)
        const refCountSum = sourceSkills.reduce((sum, skill) => sum + (skill.refCount || 0), 0)
        return {
          id: source.name,
          name: source.name,
          displayName: source.name,
          count: refCountSum,
        }
      }),
    ],
    [skills]
  )

  function mergeSources(skillsList: Skill[]) {
    const sourceMap = new Map<string, { name: string; count: number }>()
    skillsList.forEach(skill => {
      const source = skill.source || 'Unknown'
      if (!sourceMap.has(source)) {
        sourceMap.set(source, { name: source, count: 0 })
      }
      sourceMap.get(source)!.count++
    })
    return Array.from(sourceMap.values()).sort((a, b) => b.count - a.count)
  }

  const filtered = useMemo(() => {
    const q = deferredSearch.toLowerCase()
    return skills.filter(skill => {
      const matchSearch =
        skill.name.toLowerCase().includes(q) || skill.description.toLowerCase().includes(q)
      const matchClaw = selectedClaw === 'all' || skill.source === selectedClaw
      return matchSearch && matchClaw
    })
  }, [skills, deferredSearch, selectedClaw])

  const aggregatedMissingBins = useMemo(() => {
    const set = new Set<string>()
    for (const s of skills) {
      if (s.ready) continue
      for (const b of s.missingBins ?? []) set.add(b)
    }
    return [...set].sort()
  }, [skills])

  const unreadySkillCount = useMemo(() => skills.filter(s => !s.ready && s.requires.length > 0).length, [skills])

  /** 热榜项与本地技能文件夹名对齐（id / name / 仓库 URL 末尾段） */
  const localHotInstallIndex = useMemo(() => buildLocalHotInstallIndex(skills), [skills])

  function showInFinder() {
    if (!menu) return
    void invoke('show_in_finder', { path: menu.skill.path })
    setMenu(null)
  }

  async function editSkill() {
    if (!menu) return
    const skill = menu.skill
    try {
      const skillMdPath = `${skill.path}/SKILL.md`
      const agentsMdPath = `${skill.path}/AGENTS.md`
      const hasSkillMd = await invoke<boolean>('check_path_exists', { path: skillMdPath })
      const hasAgentsMd = await invoke<boolean>('check_path_exists', { path: agentsMdPath })
      const pathToOpen = hasSkillMd ? skillMdPath : hasAgentsMd ? agentsMdPath : null
      if (!pathToOpen) {
        setToast({ message: '未找到 SKILL.md 或 AGENTS.md，无法在编辑器中打开', type: 'error' })
        setMenu(null)
        return
      }
      await invoke('open_file_in_editor', { path: pathToOpen })
      setMenu(null)
    } catch (err) {
      setToast({ message: `编辑失败：${err}`, type: 'error' })
      setMenu(null)
    }
  }

  async function deleteSkill() {
    if (!menu) return
    const skill = menu.skill
    try {
      await invoke('delete_skill', { path: skill.path })
      setToast({ message: `成功删除 ${skill.name}`, type: 'success' })
      setMenu(null)
      void load()
    } catch (err) {
      setToast({ message: `删除失败：${err}`, type: 'error' })
      setMenu(null)
    }
  }

  async function installFromGithub() {
    if (!installUrl.trim()) return
    if (installing) {
      setToast({ message: `⚠️ 请等待当前安装完成`, type: 'error' })
      return
    }
    if (!claws.length) {
      setToast({
        message: '未检测到技能目录。请等待扫描完成或先安装 OpenClaw / Cursor 等以生成来源。',
        type: 'error',
      })
      return
    }

    setInstalling(true)
    setToast({ message: `📥 正在下载...`, type: 'success' })
    try {
      await invoke('install_from_github', { repoUrl: installUrl, targetPath: installTargetPath() })
      setToast({ message: `✅ 成功安装 ${installUrl}`, type: 'success' })
      setInstallUrl('')
      setActiveTab('skills')
      void load()
    } catch (err) {
      setToast({ message: `安装失败：${err}`, type: 'error' })
    } finally {
      setInstalling(false)
    }
  }

  async function addSource() {
    if (!newSourceName.trim() || !newSourcePath.trim()) return
    try {
      await invoke('add_custom_source', { name: newSourceName, path: newSourcePath })
      setToast({ message: `成功添加来源 ${newSourceName}`, type: 'success' })
      setNewSourceName('')
      setNewSourcePath('')
      setShowAddSource(false)
      void loadCustomSources()
      void load()
    } catch (err) {
      setToast({ message: `添加失败：${err}`, type: 'error' })
    }
  }

  async function removeSource(name: string) {
    try {
      await invoke('remove_custom_source', { name })
      setToast({ message: `成功移除来源 ${name}`, type: 'success' })
      void loadCustomSources()
      void load()
    } catch (err) {
      setToast({ message: `移除失败：${err}`, type: 'error' })
    }
  }

  async function doCopySkill(targetClaw: Claw, skill: Skill) {
    try {
      await invoke('copy_skill', {
        sourcePath: skill.path,
        targetPath: targetClaw.skills_path,
        skillName: skill.name,
      })
      setToast({ message: `成功复制 ${skill.name} 到 ${targetClaw.name}`, type: 'success' })
      setMenu(null)
      void load()
    } catch (err) {
      setToast({ message: `复制失败：${err}`, type: 'error' })
      setMenu(null)
    }
  }

  async function copySkillTo(targetClaw: Claw) {
    if (!menu) return
    const skill = menu.skill
    const targetSkillPath = `${targetClaw.skills_path}${skill.name}`
    const exists = await invoke<boolean>('check_path_exists', { path: targetSkillPath })
    if (exists) {
      setConfirm({ kind: 'copyOverwrite', claw: targetClaw, skill })
      return
    }
    await doCopySkill(targetClaw, skill)
  }

  const toggleSkillSelection = (skillPath: string) => {
    const newSelected = new Set(selectedSkills)
    if (newSelected.has(skillPath)) {
      newSelected.delete(skillPath)
    } else {
      newSelected.add(skillPath)
    }
    setSelectedSkills(newSelected)
  }

  const selectAllFiltered = () => {
    setSelectedSkills(new Set(filtered.map(skill => skill.path)))
  }

  const clearSelection = () => {
    setSelectedSkills(new Set())
  }

  async function batchDelete() {
    if (selectedSkills.size === 0) return
    const selectedSkillList = filtered.filter(skill => selectedSkills.has(skill.path))
    try {
      for (const skill of selectedSkillList) {
        await invoke('delete_skill', { path: skill.path })
      }
      setToast({ message: `成功删除 ${selectedSkillList.length} 个技能`, type: 'success' })
      clearSelection()
      void load()
    } catch (err) {
      setToast({ message: `批量删除失败：${err}`, type: 'error' })
    }
  }

  async function doBatchCopy(targetClaw: Claw, list: Skill[]) {
    try {
      for (const skill of list) {
        await invoke('copy_skill', {
          sourcePath: skill.path,
          targetPath: targetClaw.skills_path,
          skillName: skill.name,
        })
      }
      setToast({ message: `成功复制 ${list.length} 个技能到 ${targetClaw.name}`, type: 'success' })
      clearSelection()
      void load()
    } catch (err) {
      setToast({ message: `批量复制失败：${err}`, type: 'error' })
    }
  }

  async function batchCopyTo(targetClaw: Claw) {
    if (selectedSkills.size === 0) return
    const selectedSkillList = filtered.filter(skill => selectedSkills.has(skill.path))
    const conflicts: string[] = []
    for (const skill of selectedSkillList) {
      const p = `${targetClaw.skills_path}${skill.name}`
      if (await invoke<boolean>('check_path_exists', { path: p })) {
        conflicts.push(skill.name)
      }
    }
    if (conflicts.length > 0) {
      setConfirm({ kind: 'batchOverwrite', claw: targetClaw, conflictNames: conflicts })
      return
    }
    await doBatchCopy(targetClaw, selectedSkillList)
  }

  async function runHotInstall(hotSkill: HotSkill) {
    if (!claws.length) {
      setToast({
        message: '未检测到技能目录。请等待扫描完成或先安装 OpenClaw / Cursor 等以生成来源。',
        type: 'error',
      })
      return
    }
    setInstalling(true)
    setInstallingSkillId(hotSkill.id)
    setToast({ message: `📥 正在下载 ${hotSkill.name}...`, type: 'success' })
    try {
      await invoke('install_from_github', { repoUrl: hotSkill.github_url, targetPath: installTargetPath() })
      setToast({ message: `✅ 成功安装 ${hotSkill.name}`, type: 'success' })
      setActiveTab('skills')
      void load()
    } catch (err) {
      const errorMsg = String(err)
      if (errorMsg.includes('超时')) {
        setToast({ message: `⏱️ ${errorMsg}`, type: 'error' })
      } else if (errorMsg.includes('网络')) {
        setToast({
          message: `${errorMsg}\n\n提示：如果你使用代理，请确保已启动`,
          type: 'error',
        })
      } else if (errorMsg.includes('不存在')) {
        setToast({ message: `❌ ${errorMsg}`, type: 'error' })
      } else {
        setToast({ message: `安装失败：${err}`, type: 'error' })
      }
    } finally {
      setInstalling(false)
      setInstallingSkillId(null)
    }
  }

  function installHotSkill(hotSkill: HotSkill) {
    if (installing) {
      setToast({ message: `⚠️ 请等待当前安装完成`, type: 'error' })
      return
    }
    if (hotSkill.largeClone) {
      setConfirm({ kind: 'largeClone', hotSkill })
      return
    }
    void runHotInstall(hotSkill)
  }

  function handleConfirmOk() {
    if (!confirm) return
    if (confirm.kind === 'largeClone') {
      const h = confirm.hotSkill
      setConfirm(null)
      void runHotInstall(h)
    } else if (confirm.kind === 'copyOverwrite') {
      const { claw, skill } = confirm
      setConfirm(null)
      void doCopySkill(claw, skill)
    } else if (confirm.kind === 'batchOverwrite') {
      const { claw } = confirm
      setConfirm(null)
      const selectedSkillList = filtered.filter(skill => selectedSkills.has(skill.path))
      void doBatchCopy(claw, selectedSkillList)
    }
  }

  function handleConfirmCancel() {
    const wasCopyOverwrite = confirm?.kind === 'copyOverwrite'
    setConfirm(null)
    if (wasCopyOverwrite) setMenu(null)
  }

  async function runBrewInstall(bins: string[]) {
    if (bins.length === 0) return
    setBrewInstalling(true)
    try {
      const summary = await invoke<string>('install_bins_with_homebrew', { bins })
      const lines = summary.trim().split('\n').filter(Boolean)
      const firstLine = lines[0] ?? '安装流程已结束'
      setToast({
        message: firstLine.length > 200 ? `${firstLine.slice(0, 200)}…` : firstLine,
        type: 'success',
      })
      setBrewDialog(null)
      await load()
    } catch (err) {
      setToast({ message: String(err), type: 'error' })
    } finally {
      setBrewInstalling(false)
    }
  }

  return (
    <div className="flex h-screen min-h-0 bg-zinc-100 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100">
      <AppSidebar
        widthPx={sidebarW}
        onResizeMouseDown={onSidebarResizeMouseDown}
        activeTab={activeTab}
        onTabChange={setActiveTab}
        clawList={clawList}
        selectedClaw={selectedClaw}
        onSelectClaw={setSelectedClaw}
        hotSkills={hotSkills}
        hotPlatform={hotPlatform}
        onHotPlatform={setHotPlatform}
        hotCategory={hotCategory}
        onHotCategory={setHotCategory}
        customSources={customSources}
        onRemoveCustomSource={removeSource}
        onShowAddSource={() => setShowAddSource(true)}
      />

      <div className="min-h-0 min-w-0 flex-1 overflow-auto p-6">
        <div className="mb-6 flex min-h-[40px] flex-wrap items-center justify-between gap-2">
          {activeTab === 'skills' && (
            <>
              <div className="relative max-w-md min-w-0 flex-1">
                <Search className="pointer-events-none absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-zinc-400 dark:text-zinc-500" />
                <input
                  ref={searchInputRef}
                  type="search"
                  placeholder="搜索 skill…（按 / 聚焦）"
                  value={search}
                  onChange={e => setSearch(e.target.value)}
                  className="w-full rounded-xl border border-zinc-300 bg-white py-3 pl-10 pr-4 text-zinc-900 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:placeholder-zinc-500"
                  aria-label="搜索技能"
                />
              </div>
              <div className="flex shrink-0 flex-wrap items-center gap-2">
                {aggregatedMissingBins.length > 0 && !loading && (
                  <button
                    type="button"
                    onClick={() => setBrewDialog({ bins: aggregatedMissingBins })}
                    className="flex items-center gap-2 rounded-xl border border-amber-500/50 bg-amber-100 px-4 py-2 text-sm font-medium text-amber-950 hover:bg-amber-200 dark:border-amber-700/50 dark:bg-amber-950/50 dark:text-amber-100 dark:hover:bg-amber-900/60"
                    title="使用 Homebrew 安装 PATH 中缺失的命令（常见 bin 已映射到 formula）"
                    aria-label={`补齐命令行依赖，共 ${aggregatedMissingBins.length} 项`}
                  >
                    <Terminal className="h-5 w-5 shrink-0" />
                    补齐依赖 ({aggregatedMissingBins.length})
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => load()}
                  className="flex items-center gap-2 rounded-xl border border-zinc-300 bg-white px-4 py-2 text-zinc-900 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700"
                >
                  <RefreshCw className="h-5 w-5" /> 刷新
                </button>
                <button
                  type="button"
                  onClick={() => setActiveTab('sources')}
                  className="flex items-center gap-2 rounded-xl border border-zinc-300 bg-white px-4 py-2 text-zinc-900 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700"
                >
                  <FolderOpen className="h-5 w-5" /> 来源管理
                </button>
              </div>
            </>
          )}

          {activeTab === 'hot' && (
            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
              <h2 className="flex items-center gap-2 text-xl font-bold">
                <Flame className="h-6 w-6 shrink-0 text-orange-500" /> 社区热榜
              </h2>
              <button
                type="button"
                onClick={() => loadHotSkills()}
                className="rounded-xl bg-zinc-200 px-4 py-2 hover:bg-zinc-300 dark:bg-zinc-800 dark:hover:bg-zinc-700"
                title="刷新（可用缓存）"
                aria-label="刷新热榜"
              >
                <RefreshCw className="h-5 w-5" />
              </button>
              <button
                type="button"
                onClick={() => refreshHotSkillsClearCache()}
                className="rounded-xl border border-amber-700/40 bg-amber-100/90 px-3 py-2 text-sm text-amber-950 hover:bg-amber-200/90 dark:border-amber-800/50 dark:bg-amber-950/40 dark:text-amber-100 dark:hover:bg-amber-900/55"
                title="清除热榜缓存并重新拉取"
              >
                强制刷新
              </button>
              {claws.length > 0 && (
                <label className="ml-auto flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
                  <span className="shrink-0">安装到</span>
                  <select
                    value={installTargetId}
                    onChange={e => setInstallTargetId(e.target.value)}
                    className="max-w-[200px] rounded-lg border border-zinc-300 bg-white px-2 py-1.5 text-zinc-900 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-100"
                    aria-label="热榜安装目标目录"
                  >
                    {claws.map(c => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </label>
              )}
            </div>
          )}

          {activeTab === 'sources' && (
            <h2 className="flex flex-1 items-center gap-2 text-xl font-bold">
              <FolderOpen className="h-6 w-6" /> 来源管理
            </h2>
          )}
        </div>

        {activeTab === 'skills' && aggregatedMissingBins.length > 0 && !loading && (
          <div
            className="mb-4 flex flex-wrap items-center gap-3 rounded-xl border border-amber-200 bg-amber-50/95 px-4 py-3 text-sm dark:border-amber-900/60 dark:bg-amber-950/35"
            role="status"
          >
            <span className="min-w-0 flex-1 text-amber-950 dark:text-amber-100">
              有 <strong>{unreadySkillCount}</strong> 个技能声明了命令行依赖但未就绪，共可尝试安装{' '}
              <strong>{aggregatedMissingBins.length}</strong> 个缺失命令（已去重）。需要本机已安装{' '}
              <a
                href="https://brew.sh"
                target="_blank"
                rel="noreferrer"
                className="underline hover:no-underline"
              >
                Homebrew
              </a>
              。
            </span>
            <button
              type="button"
              onClick={() => setBrewDialog({ bins: aggregatedMissingBins })}
              disabled={brewInstalling}
              className="shrink-0 rounded-lg bg-amber-600 px-4 py-2 font-medium text-white hover:bg-amber-700 disabled:cursor-not-allowed disabled:opacity-60"
            >
              一键尝试 Homebrew 安装
            </button>
          </div>
        )}

        {activeTab === 'skills' && (
          <div className="mb-6 flex flex-col gap-2 sm:flex-row sm:items-center">
            {claws.length > 0 && (
              <label className="flex shrink-0 items-center gap-2 text-sm text-zinc-600 dark:text-zinc-400">
                <span>安装到</span>
                <select
                  value={installTargetId}
                  onChange={e => setInstallTargetId(e.target.value)}
                  className="rounded-lg border border-zinc-300 bg-white px-2 py-2 text-zinc-900 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-100"
                  aria-label="在线安装目标目录"
                >
                  {claws.map(c => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </label>
            )}
            <input
              type="text"
              placeholder="GitHub 仓库地址 (如：author/repo 或 https://github.com/author/repo)"
              value={installUrl}
              onChange={e => setInstallUrl(e.target.value)}
              className="min-w-0 flex-1 rounded-xl border border-zinc-300 bg-white px-4 py-3 text-zinc-900 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:placeholder-zinc-500"
            />
            <button
              type="button"
              onClick={installFromGithub}
              disabled={installing || !installUrl.trim()}
              className="flex items-center justify-center gap-2 rounded-xl bg-emerald-600 px-6 py-3 text-white hover:bg-emerald-700 disabled:cursor-not-allowed disabled:bg-zinc-300 dark:disabled:bg-zinc-700"
            >
              {installing ? <RefreshCw className="h-5 w-5 animate-spin" /> : <Download className="h-5 w-5" />}
              在线安装
            </button>
          </div>
        )}

        {selectedSkills.size > 0 && activeTab === 'skills' && (
          <div className="mb-4 flex flex-wrap items-center gap-4 rounded-lg bg-zinc-200/60 p-3 dark:bg-zinc-800/50">
            <span className="text-sm text-zinc-600 dark:text-zinc-300">已选择 {selectedSkills.size} 个技能</span>
            <button type="button" onClick={selectAllFiltered} className="text-sm text-blue-400 hover:text-blue-300">
              全选
            </button>
            <button type="button" onClick={clearSelection} className="text-sm text-zinc-500 hover:text-zinc-600 dark:text-zinc-400 dark:hover:text-zinc-300">
              取消选择
            </button>
            <div className="flex flex-1 flex-wrap items-center justify-end gap-2">
              <select
                className="rounded border border-zinc-300 bg-zinc-200 px-2 py-1 text-sm dark:border-zinc-600 dark:bg-zinc-700"
                aria-label="批量复制到来源"
                defaultValue=""
                onChange={e => {
                  if (e.target.value) {
                    const targetClaw = claws.find(c => c.name === e.target.value)
                    if (targetClaw) void batchCopyTo(targetClaw)
                    e.target.value = ''
                  }
                }}
              >
                <option value="">复制到...</option>
                {claws.map(claw => (
                  <option key={claw.name} value={claw.name}>
                    {claw.name}
                  </option>
                ))}
              </select>
              <button type="button" onClick={batchDelete} className="rounded bg-red-600 px-3 py-1 text-sm hover:bg-red-700">
                批量删除
              </button>
            </div>
          </div>
        )}

        {loading && activeTab === 'skills' ? (
          <div className="py-20 text-center text-zinc-500 dark:text-zinc-400">加载中...</div>
        ) : activeTab === 'hot' ? (
          hotLoading ? (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {Array.from({ length: 6 }).map((_, i) => (
                <div
                  key={i}
                  className="h-52 animate-pulse rounded-xl border border-zinc-200 bg-zinc-200/80 dark:border-zinc-700 dark:bg-zinc-800/80"
                />
              ))}
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {(() => {
                const platformFiltered =
                  hotPlatform === 'all'
                    ? hotSkills
                    : hotSkills.filter(s => resolveHotSkillPlatform(s) === hotPlatform)

                const filteredHotSkills =
                  hotCategory === 'all'
                    ? platformFiltered
                    : hotCategory === '__other__'
                      ? platformFiltered.filter(s => !(s.tags && s.tags.length))
                      : platformFiltered.filter(s => s.tags.includes(hotCategory))

                if (filteredHotSkills.length === 0) {
                  return (
                    <div className="col-span-full py-20 text-center text-zinc-500 dark:text-zinc-400">
                      <div className="mb-4 text-4xl">📭</div>
                      <div>该分类下暂无技能</div>
                    </div>
                  )
                }

                return filteredHotSkills.map((skill, index) => {
                  const hotInstalled = isHotSkillInstalledLocally(skill, localHotInstallIndex)
                  return (
                  <div
                    key={skill.id}
                    className={`rounded-xl border bg-white p-4 shadow-sm dark:bg-zinc-800 ${
                      hotInstalled
                        ? 'border-emerald-500/40 ring-1 ring-emerald-500/25 hover:border-emerald-500/50 dark:border-emerald-700/50'
                        : 'border-zinc-200 hover:border-zinc-300 dark:border-zinc-700 dark:hover:border-zinc-600'
                    }`}
                  >
                    <div className="mb-2 flex items-start justify-between gap-3">
                      <div className="flex min-w-0 flex-1 items-center gap-2">
                        <span className="shrink-0 text-2xl">{skill.emoji}</span>
                        <div className="min-w-0">
                          <div className="flex min-w-0 flex-wrap items-center gap-2">
                            <h3 className="font-semibold">{skill.name}</h3>
                            {hotInstalled && (
                              <span className="shrink-0 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-400">
                                已安装
                              </span>
                            )}
                          </div>
                          <p className="text-xs text-zinc-500 dark:text-zinc-500">@{skill.author}</p>
                        </div>
                      </div>
                      <div className="flex shrink-0 flex-col items-end gap-2">
                        <HotSkillPlatformBadge skill={skill} />
                        <div className="text-xs text-zinc-500 dark:text-zinc-400">#{index + 1}</div>
                        <div className="flex items-center gap-1 text-xs text-yellow-400">
                          <span>⭐</span> {skill.stars}
                        </div>
                      </div>
                    </div>
                    <p className="mb-3 line-clamp-2 text-sm text-zinc-500 dark:text-zinc-400">{skill.description}</p>
                    {skill.largeClone && (
                      <p className="mb-2 text-xs text-amber-400/90">大型仓库：克隆可能较慢、磁盘占用较高</p>
                    )}
                    <div className="flex items-center justify-between">
                      <div className="flex flex-wrap gap-1">
                        {skill.tags.map(tag => (
                          <span key={tag} className="rounded bg-zinc-200 px-2 py-0.5 text-xs dark:bg-zinc-700">
                            {tag}
                          </span>
                        ))}
                      </div>
                      <div className="text-xs text-zinc-500 dark:text-zinc-500">{skill.installs} 次安装</div>
                    </div>
                    <button
                      type="button"
                      onClick={() => installHotSkill(skill)}
                      disabled={installing || hotInstalled}
                      className={`mt-3 flex w-full items-center justify-center gap-2 rounded-lg py-2 text-sm ${
                        hotInstalled
                          ? 'cursor-default bg-emerald-500/20 text-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300'
                          : installingSkillId === skill.id
                            ? 'bg-yellow-600 text-white'
                            : installing
                              ? 'cursor-not-allowed bg-zinc-300 text-zinc-500 dark:bg-zinc-700 dark:text-zinc-400'
                              : 'bg-green-600 text-white hover:bg-green-700'
                      }`}
                    >
                      {hotInstalled ? (
                        <>
                          <CheckCircle className="h-4 w-4 shrink-0" /> 已安装
                        </>
                      ) : installingSkillId === skill.id ? (
                        <>
                          <RefreshCw className="h-4 w-4 animate-spin" /> 安装中...
                        </>
                      ) : installing ? (
                        <>
                          <RefreshCw className="h-4 w-4" /> 请等待...
                        </>
                      ) : (
                        <>
                          <Download className="h-4 w-4" /> 安装
                        </>
                      )}
                    </button>
                  </div>
                )
                })
              })()}
            </div>
          )
        ) : activeTab === 'sources' ? (
          <div className="space-y-4">
            {showAddSource && (
              <div className="rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-700 dark:bg-zinc-800">
                <h3 className="mb-3 font-semibold">添加自定义来源</h3>
                <div className="space-y-3">
                  <input
                    type="text"
                    placeholder="来源名称"
                    value={newSourceName}
                    onChange={e => setNewSourceName(e.target.value)}
                    className="w-full rounded-lg border border-zinc-300 bg-zinc-50 px-4 py-2 text-zinc-900 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                  <input
                    type="text"
                    placeholder="路径 (如：/Users/xxx/skills)"
                    value={newSourcePath}
                    onChange={e => setNewSourcePath(e.target.value)}
                    className="w-full rounded-lg border border-zinc-300 bg-zinc-50 px-4 py-2 text-zinc-900 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                  <div className="flex gap-2">
                    <button type="button" onClick={addSource} className="rounded-lg bg-green-600 px-4 py-2 hover:bg-green-700">
                      添加
                    </button>
                    <button type="button" onClick={() => setShowAddSource(false)} className="rounded-lg bg-zinc-200 px-4 py-2 hover:bg-zinc-300 dark:bg-zinc-700 dark:hover:bg-zinc-600">
                      取消
                    </button>
                  </div>
                </div>
              </div>
            )}

            <p className="mb-2 text-sm text-zinc-600 dark:text-zinc-400">
              自动发现的目录与下方「自定义」配置会合并参与扫描；同一 skills 路径不会重复添加。
            </p>
            <div className="mb-2 text-sm text-zinc-500 dark:text-zinc-400">自动发现</div>
            {claws
              .filter(c => !c.id.startsWith('custom:'))
              .map(claw => (
                <div
                  key={claw.id}
                  className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-700 dark:bg-zinc-800"
                >
                  <div>
                    <div className="font-semibold">{claw.name}</div>
                    <div className="text-sm text-zinc-500 dark:text-zinc-500">{claw.skills_path}</div>
                  </div>
                  <div className="text-sm text-zinc-500 dark:text-zinc-400">{skills.filter(s => s.source === claw.name).length} 个技能</div>
                </div>
              ))}
            <div className="mb-2 mt-4 text-sm text-zinc-500 dark:text-zinc-400">自定义（可删除配置项）</div>
            {customSources.map(source => (
              <div key={source.name} className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-700 dark:bg-zinc-800">
                <div>
                  <div className="flex items-center gap-2 font-semibold">
                    {source.name}
                    <span className="rounded bg-blue-600 px-2 py-0.5 text-xs text-white">自定义</span>
                  </div>
                  <div className="text-sm text-zinc-500 dark:text-zinc-500">{source.path}</div>
                </div>
                <button type="button" onClick={() => removeSource(source.name)} className="p-2 text-red-400 hover:text-red-300" aria-label={`删除来源 ${source.name}`}>
                  <Trash2 className="h-5 w-5" />
                </button>
              </div>
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className="py-20 text-center text-zinc-500 dark:text-zinc-400">
            <div className="mb-4 text-4xl">📦</div>
            <div>没有找到 Skills</div>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {filtered.map(skill => {
              const installBadgeCount = Math.max(skill.refCount ?? 0, skill.otherClawCount ?? 0)
              const badgePlatform =
                selectedClaw === 'all' || selectedClaw === '全部' ? skill.source : selectedClaw
              return (
                <div
                  key={skill.path}
                  onContextMenu={e => {
                    e.preventDefault()
                    void openSkillContextMenu(e.clientX, e.clientY, skill)
                  }}
                  onClick={() => setDetailSkill(skill)}
                  className="relative cursor-pointer rounded-xl border border-zinc-200 bg-white p-4 shadow-sm hover:border-zinc-300 dark:border-zinc-700 dark:bg-zinc-800 dark:hover:border-zinc-600"
                >
                  <div
                    className="absolute right-2 top-2 flex h-5 w-5 cursor-pointer items-center justify-center rounded-full border border-zinc-300 bg-white/90 hover:bg-zinc-100 dark:border-zinc-600 dark:bg-zinc-900/80 dark:hover:bg-zinc-700"
                    onClick={e => {
                      e.stopPropagation()
                      toggleSkillSelection(skill.path)
                    }}
                    role="checkbox"
                    aria-checked={selectedSkills.has(skill.path)}
                    aria-label={`选择 ${skill.name}`}
                  >
                    {selectedSkills.has(skill.path) && <div className="h-3 w-3 rounded-full bg-green-500" />}
                  </div>
                  <div className="mb-2 flex items-center gap-2">
                    <span className="text-2xl">{skill.emoji}</span>
                    <h3 className="font-semibold">{skill.name}</h3>
                  </div>
                  <p className="mb-3 line-clamp-2 text-sm text-zinc-500 dark:text-zinc-400">{skill.description}</p>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1">
                      {skill.ready ? <CheckCircle className="h-4 w-4 text-green-500" /> : <XCircle className="h-4 w-4 text-red-500" />}
                      <span className="text-xs text-zinc-500 dark:text-zinc-500">{skill.source}</span>
                    </div>
                    {installBadgeCount > 0 && (
                      <div className="inline-flex items-center gap-1.5 rounded-full bg-zinc-200/90 px-2 py-1 text-xs dark:bg-white/10">
                        <div className="h-2 w-2 rounded-full" style={{ backgroundColor: getSourceColor(badgePlatform) }} />
                        <span className="font-medium text-zinc-900 dark:text-white">{badgePlatform}</span>
                        <span className="font-semibold text-green-400">+{installBadgeCount}</span>
                      </div>
                    )}
                  </div>
                  {!skill.ready && (skill.missingBins?.length ?? 0) > 0 && (
                    <div className="mt-3 border-t border-zinc-200 pt-3 dark:border-zinc-700">
                      <p className="mb-2 line-clamp-2 text-left text-xs text-amber-700 dark:text-amber-400" title={skill.missingBins!.join('、')}>
                        缺 {skill.missingBins!.length} 项：
                        {skill.missingBins!.join('、')}
                      </p>
                      <button
                        type="button"
                        disabled={brewInstalling}
                        onClick={e => {
                          e.stopPropagation()
                          setBrewDialog({ bins: [...skill.missingBins!] })
                        }}
                        className="flex w-full items-center justify-center gap-2 rounded-lg bg-amber-600 px-3 py-2 text-xs font-medium text-white hover:bg-amber-700 disabled:cursor-not-allowed disabled:opacity-60"
                        aria-label={`为 ${skill.name} 安装缺失命令行依赖：${skill.missingBins!.join('、')}`}
                      >
                        <Terminal className="h-3.5 w-3.5 shrink-0" />
                        {brewInstalling ? '安装进行中…' : '安装本技能依赖'}
                      </button>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}
      </div>

      {detailSkill && (
        <SkillDetailModal
          skill={detailSkill}
          onClose={() => setDetailSkill(null)}
          brewBusy={brewInstalling}
          onBrewFix={
            !detailSkill.ready && (detailSkill.missingBins?.length ?? 0) > 0
              ? () => setBrewDialog({ bins: detailSkill.missingBins! })
              : undefined
          }
        />
      )}

      {menu && (
        <SkillContextMenu
          menu={menu}
          claws={claws}
          selectedClaw={selectedClaw}
          clawExistsMap={clawExistsMap}
          onCopyTo={copySkillTo}
          onShowInFinder={showInFinder}
          onEdit={editSkill}
          onDelete={deleteSkill}
          onDismiss={() => setMenu(null)}
        />
      )}

      <ConfirmDialog
        open={confirm?.kind === 'largeClone'}
        title="大型仓库"
        message="此仓库体积较大，浅克隆仍可能较慢且占用较多磁盘空间，确定要继续安装吗？"
        confirmLabel="继续安装"
        onConfirm={handleConfirmOk}
        onCancel={handleConfirmCancel}
      />

      <ConfirmDialog
        open={confirm?.kind === 'copyOverwrite'}
        title="覆盖已存在的技能？"
        message={
          confirm?.kind === 'copyOverwrite'
            ? `「${confirm.skill.name}」在「${confirm.claw.name}」下已存在，是否覆盖？`
            : ''
        }
        danger
        confirmLabel="覆盖"
        onConfirm={handleConfirmOk}
        onCancel={handleConfirmCancel}
      />

      <ConfirmDialog
        open={confirm?.kind === 'batchOverwrite'}
        title="覆盖已存在的技能？"
        message={
          confirm?.kind === 'batchOverwrite'
            ? `以下技能在「${confirm.claw.name}」下已存在：\n${confirm.conflictNames.join('、')}\n\n确定要全部覆盖吗？`
            : ''
        }
        danger
        confirmLabel="全部覆盖"
        onCancel={handleConfirmCancel}
        onConfirm={handleConfirmOk}
      />

      <ConfirmDialog
        open={brewDialog !== null}
        title="用 Homebrew 补齐命令行依赖？"
        message={
          brewDialog
            ? `将执行 brew install（formula 已根据常见命令名做映射并去重）。\n\n缺失命令：${brewDialog.bins.join('、')}\n\n需要网络；若应用从访达启动，PATH 可能与终端不一致，本应用已尝试附带常见 Homebrew 路径。若失败请在终端手动安装。`
            : ''
        }
        confirmLabel={brewInstalling ? '正在安装…' : '开始安装'}
        busy={brewInstalling}
        onConfirm={() => brewDialog && void runBrewInstall(brewDialog.bins)}
        onCancel={() => {
          if (!brewInstalling) setBrewDialog(null)
        }}
      />

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  )
}

export default App
