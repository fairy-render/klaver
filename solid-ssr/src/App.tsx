import { createResource, createSignal, Suspense } from 'solid-js'

// import './App.css'

function App() {
    const [count, setCount] = createSignal(0)



    return (
        <>
            <section id="center">
                {/* <Suspense><Inner /></Suspense> */}
                <div>
                    <h1>Get started</h1>
                    <p>
                        Edit <code>src/App.tsx</code> and save to test <code>HMR</code>
                    </p>
                </div>
                <button class="counter" onClick={() => setCount((count) => count + 1)}>
                    Count is {count()}
                </button>
            </section>

            <div class="ticks"></div>

            <section id="next-steps">
                <div id="docs">

                    <h2>Documentation</h2>
                    <p>Your questions, answered</p>
                    <ul>
                        <li>
                            <a href="https://vite.dev/" target="_blank">

                                Explore Vite
                            </a>
                        </li>
                        <li>
                            <a href="https://solidjs.com/" target="_blank">
                                Learn more
                            </a>
                        </li>
                    </ul>
                </div>
                <div id="social">

                    <h2>Connect with us</h2>
                    <p>Join the Vite community</p>
                    <ul>
                        <li>
                            <a href="https://github.com/vitejs/vite" target="_blank">

                                GitHub
                            </a>
                        </li>
                        <li>
                            <a href="https://chat.vite.dev/" target="_blank">

                                Discord
                            </a>
                        </li>
                        <li>
                            <a href="https://x.com/vite_js" target="_blank">

                                X.com
                            </a>
                        </li>
                        <li>
                            <a href="https://bsky.app/profile/vite.dev" target="_blank">

                                Bluesky
                            </a>
                        </li>
                    </ul>
                </div>
            </section>

            <div class="ticks"></div>
            <section id="spacer"></section>
        </>
    )
}

function Inner() {
    const [res] = createResource(() => {
        return fetch('https://jsonplaceholder.typicode.com/todos/1').then(res => res.json())
    })

    return <div>{res() ? <pre>{JSON.stringify(res(), null, 2)}</pre> : 'Loading...'}</div>
}

export default App