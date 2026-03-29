import { describe, expect, it } from 'vitest'
import { buildLocalHotInstallIndex, isHotSkillInstalledLocally } from './hotSkillLocalMatch'
import type { HotSkill } from '../hotTypes'
import type { Skill } from '../types/domain'

function mockSkill(partial: Partial<Skill> & Pick<Skill, 'name' | 'path'>): Skill {
  return {
    description: '',
    emoji: '📦',
    requires: [],
    ready: true,
    source: 'test',
    tags: [],
    ...partial,
  }
}

function mockHot(partial: Partial<HotSkill> & Pick<HotSkill, 'id' | 'name' | 'github_url'>): HotSkill {
  return {
    description: '',
    emoji: '✨',
    author: 'test',
    stars: 0,
    installs: 0,
    tags: [],
    ...partial,
  }
}

describe('buildLocalHotInstallIndex', () => {
  it('indexes skill name and path leaf lowercased', () => {
    const idx = buildLocalHotInstallIndex([
      mockSkill({ name: 'My-Skill', path: '/skills/clones/my-skill' }),
    ])
    expect(idx.has('my-skill')).toBe(true)
  })
})

describe('isHotSkillInstalledLocally', () => {
  it('returns true when hot id matches local folder name', () => {
    const idx = buildLocalHotInstallIndex([mockSkill({ name: 'ontology', path: '/x/ontology' })])
    const hot = mockHot({
      id: 'ontology',
      name: 'Ontology',
      github_url: 'https://github.com/oswalpalash/ontology',
    })
    expect(isHotSkillInstalledLocally(hot, idx)).toBe(true)
  })

  it('returns true when repo basename matches even if hot name differs', () => {
    const idx = buildLocalHotInstallIndex([mockSkill({ name: 'api-gateway', path: '/a/api-gateway' })])
    const hot = mockHot({
      id: 'api-gateway',
      name: 'Different Title',
      github_url: 'https://github.com/byungkyu/api-gateway',
    })
    expect(isHotSkillInstalledLocally(hot, idx)).toBe(true)
  })

  it('returns false when no overlap', () => {
    const idx = buildLocalHotInstallIndex([mockSkill({ name: 'foo', path: '/a/foo' })])
    const hot = mockHot({
      id: 'bar',
      name: 'bar',
      github_url: 'https://github.com/x/bar',
    })
    expect(isHotSkillInstalledLocally(hot, idx)).toBe(false)
  })
})
