import { useEffect, useState } from "react"
import reactLogo from "./assets/react.svg"
import { invoke } from "@tauri-apps/api/tauri"
import "./App.css"
import { emit, listen } from "@tauri-apps/api/event"

function App() {
  const [last_clipboard, setLastClipboard] = useState("")
  const [lastMousePosition, setLastMousePosition] = useState([0, 0])
  const [isSelecting, setIsSelecting] = useState(false)

  async function getClipboard() {
    return await invoke("get_clipboard")
  }

  async function copyToClipboard() {
    await invoke("copy_to_clipboard")
  }

  async function setClipboard(text) {
    await invoke("set_clipboard", { text })
  }

  async function openInDefaultBrowser(url) {
    await invoke("open_url_in_default_browser", { url })
  }

  useEffect(() => {
    setClipboard("").then()
  }, [])

  useEffect(() => {
    const unlistenClick = listen("left-click", async (event) => {
      let location = event.payload

      setLastMousePosition(() => [location.x, location.y])

      return () => {
        unlistenClick.then((unlisten) => unlisten())
      }
    })
    const unlistenClickRelease = listen("left-click-release", async (event) => {
      // setTimeout(() => {}, 100)
      let location = event.payload

      if (
        (lastMousePosition[0] - location.x) ** 2 +
          (lastMousePosition[1] - location.y) ** 2 >
        5
      ) {
        setIsSelecting(true)
        setLastMousePosition(() => [location.x, location.y])
      }
    })

    return () => {
      unlistenClickRelease.then((unlisten) => unlisten())
    }
  })

  // useEffect(() => {
  //   if (last_clipboard) {
  //     openInDefaultBrowser(`http://www.google.com/search?q=${last_clipboard}`)
  //   }
  // }, [last_clipboard])

  useEffect(() => {
    if (isSelecting) {
      let func = async () => {
        await copyToClipboard()
        const clipboard = await getClipboard()
        if (clipboard && clipboard !== last_clipboard) {
          emit("selecting", lastMousePosition)
          await setClipboard(last_clipboard)
          setLastClipboard(clipboard)
        }
        setIsSelecting(false)
      }
      func().then()
    } else {
      // emit("not-selecting", {})
    }
  })

  return (
    <div className="container">
      {/* <p>{last_clipboard}</p> */}
      <button
        onClick={() => {
          openInDefaultBrowser(
            `http://www.google.com/search?q=${last_clipboard}`
          )

          emit("not-selecting", {})
        }}
      >
        Open in Google
      </button>
    </div>
  )
}

export default App
