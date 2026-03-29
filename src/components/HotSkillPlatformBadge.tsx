import type { HotSkill } from '../hotTypes'
import { PLATFORM_BADGE_STYLE, resolveHotSkillPlatform } from '../hotTypes'

export function HotSkillPlatformBadge({ skill }: { skill: HotSkill }) {
  const plat = resolveHotSkillPlatform(skill)
  const st = PLATFORM_BADGE_STYLE[plat]
  return (
    <div
      className={`flex items-center gap-1.5 pl-1.5 pr-2 py-0.5 rounded-md border shrink-0 ${st.box}`}
      title={st.label}
    >
      <img
        src={st.src}
        alt=""
        width={20}
        height={20}
        className="w-5 h-5 object-contain rounded opacity-90"
      />
      <span className={`text-xs font-medium ${st.text}`}>{st.label}</span>
    </div>
  )
}
