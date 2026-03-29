import React from 'react'
import ReactDOM from 'react-dom/client'
import { isTauri } from '@tauri-apps/api/core'
import App from './App'
import './index.css'

/**
 * Tauri / WKWebView 默认右键菜单含 Reload 等，易被当成应用 bug。
 * 在捕获阶段 cancel 默认行为；自定义菜单仍依赖各元素上的 onContextMenu（仍会冒泡到 React）。
 */
if (isTauri()) {
  document.addEventListener(
    'contextmenu',
    e => {
      e.preventDefault()
    },
    { capture: true }
  )
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
