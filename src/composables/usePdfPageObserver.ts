import { onUnmounted, type Ref, type ShallowRef, watch } from 'vue'
import type { PDFDocumentProxy } from 'pdfjs-dist'

interface PageRenderState {
  rendered: boolean
  rendering: boolean
}

/**
 * Composable that lazily renders PDF pages via IntersectionObserver.
 * Each observed canvas is rendered when it enters the root element's viewport
 * (plus the given rootMargin buffer).
 *
 * Used twice in PdfViewer: once for the main view and once for thumbnails.
 */
export function usePdfPageObserver(
  pdfDoc: Ref<PDFDocumentProxy | null> | ShallowRef<PDFDocumentProxy | null>,
  scale: number,
  rootMargin = '200px 0px',
) {
  let observer: IntersectionObserver | null = null
  const canvasMap = new Map<Element, number>() // canvas → pageNum
  const renderState = new Map<number, PageRenderState>()

  function createObserver(root: HTMLElement) {
    reset()
    observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue
          const pageNum = canvasMap.get(entry.target)
          if (pageNum == null) continue
          renderPageToCanvas(pageNum, entry.target as HTMLCanvasElement)
        }
      },
      { root, rootMargin, threshold: 0 },
    )

    // Re-observe any canvases already registered
    for (const [canvas] of canvasMap) {
      observer.observe(canvas)
    }
  }

  function observe(canvas: HTMLCanvasElement, pageNum: number) {
    canvasMap.set(canvas, pageNum)
    renderState.set(pageNum, { rendered: false, rendering: false })
    observer?.observe(canvas)
  }

  function unobserve(canvas: HTMLCanvasElement) {
    const pageNum = canvasMap.get(canvas)
    observer?.unobserve(canvas)
    canvasMap.delete(canvas)
    if (pageNum != null) renderState.delete(pageNum)
  }

  async function renderPageToCanvas(pageNum: number, canvas: HTMLCanvasElement) {
    const doc = pdfDoc.value
    if (!doc) return

    const state = renderState.get(pageNum)
    if (!state || state.rendered || state.rendering) return
    state.rendering = true

    try {
      const page = await doc.getPage(pageNum)
      const viewport = page.getViewport({ scale })
      canvas.width = viewport.width
      canvas.height = viewport.height
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      await page.render({ canvasContext: ctx, viewport, canvas }).promise
      state.rendered = true
    } catch (err) {
      console.error(`Failed to render page ${pageNum}:`, err)
    } finally {
      state.rendering = false
    }
  }

  function reset() {
    observer?.disconnect()
    observer = null
    canvasMap.clear()
    renderState.clear()
  }

  // When the pdfDoc changes, reset render states so pages re-render
  watch(pdfDoc, () => {
    for (const [, state] of renderState) {
      state.rendered = false
      state.rendering = false
    }
  })

  onUnmounted(() => {
    reset()
  })

  return { createObserver, observe, unobserve, reset }
}
