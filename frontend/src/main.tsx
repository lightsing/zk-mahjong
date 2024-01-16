import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'

// @ts-expect-error no def in it's .d.ts
import { __tla } from 'zk-mahjong-wasm'

__tla.then(() => {
    ReactDOM.createRoot(document.getElementById('root')!).render(
        <React.StrictMode>
            <App />
        </React.StrictMode>
    )
})
