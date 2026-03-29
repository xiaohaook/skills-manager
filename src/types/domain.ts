/** 与后端 / invoke 约定对齐的领域类型 */

export interface Skill {
  name: string
  description: string
  emoji: string
  path: string
  requires: string[]
  /** PATH 中当前未找到的可执行名（`bins` 的子集） */
  missingBins?: string[]
  ready: boolean
  source: string
  tags: string[]
  refCount?: number
  otherClawCount?: number
}

export interface Claw {
  id: string
  name: string
  skills_path: string
  is_local: boolean
}

export interface SourceConfig {
  name: string
  path: string
  enabled: boolean
}
