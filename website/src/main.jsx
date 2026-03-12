import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.jsx'

const base = import.meta.env.BASE_URL || '/'
const pathname = window.location.pathname
const generetPath = `${base}generet/`
const generetPathNoSlash = generetPath.endsWith('/')
  ? generetPath.slice(0, -1)
  : generetPath

// Vite dev serves SPA fallback for /polyws/generet/, so redirect to static page.
if (pathname === generetPath || pathname === generetPathNoSlash) {
  window.location.replace(`${base}generet/index.html`)
} else {
  createRoot(document.getElementById('root')).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
}
