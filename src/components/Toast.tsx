import { useEffect } from 'react'
import { X } from 'lucide-react'

export function Toast({
  message,
  type,
  onClose,
}: {
  message: string
  type: 'success' | 'error'
  onClose: () => void
}) {
  useEffect(() => {
    const timer = setTimeout(onClose, 4000)
    return () => clearTimeout(timer)
  }, [onClose])

  return (
    <div
      role="alert"
      className={`fixed bottom-4 right-4 z-[60] flex max-w-[min(420px,calc(100vw-2rem))] items-center gap-2 rounded-lg px-4 py-3 text-sm shadow-lg ${
        type === 'success' ? 'bg-green-600 text-white' : 'bg-red-600 text-white'
      }`}
    >
      <span className="min-w-0 flex-1 whitespace-pre-wrap break-words">{message}</span>
      <button
        type="button"
        aria-label="关闭通知"
        onClick={onClose}
        className="shrink-0 rounded p-1 hover:bg-white/20"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  )
}
