import { renderToString, generateHydrationScript, renderToStringAsync } from 'solid-js/web'
import App from './App'

/**
 * @param {string} _url
 */
export async function render(_url) {
    const html = await renderToStringAsync(() => <App />)
    const hydration = generateHydrationScript()
    return { html, hydration }
}