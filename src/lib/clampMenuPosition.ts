/** 将右键菜单左上角约束在视口内（估算宽高，避免裁切） */
export function clampMenuPosition(
  clientX: number,
  clientY: number,
  menuWidth = 240,
  menuHeight = 360
): { left: number; top: number } {
  const pad = 8
  const vw = typeof window !== 'undefined' ? window.innerWidth : 1200
  const vh = typeof window !== 'undefined' ? window.innerHeight : 800
  const maxX = Math.max(pad, vw - menuWidth - pad)
  const maxY = Math.max(pad, vh - menuHeight - pad)
  return {
    left: Math.min(Math.max(pad, clientX), maxX),
    top: Math.min(Math.max(pad, clientY), maxY),
  }
}
