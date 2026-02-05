<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Progress } from '@/components/ui/progress'
import { useConnectionStore, useDocumentsStore, useIngestStore, useUiStore } from '@/stores'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const ingestStore = useIngestStore()
const uiStore = useUiStore()

type IngestMode = 'pdf' | 'images'

const mode = ref<IngestMode>('pdf')
const selectedFile = ref<string | null>(null)
const selectedFiles = ref<string[]>([])
const titleOverride = ref('')
const authorOverride = ref('')
const documentTitle = ref('')

const fileName = computed(() => {
  if (!selectedFile.value) return ''
  const parts = selectedFile.value.split('/')
  return parts[parts.length - 1] || ''
})

const selectedFileNames = computed(() => {
  return selectedFiles.value.map((path) => {
    const parts = path.split('/')
    return parts[parts.length - 1] || path
  })
})

const canIngestPdf = computed(() => {
  return (
    selectedFile.value &&
    connectionStore.isConnected &&
    !ingestStore.isIngesting
  )
})

const canIngestImages = computed(() => {
  return (
    selectedFiles.value.length > 0 &&
    documentTitle.value.trim() !== '' &&
    connectionStore.isConnected &&
    !ingestStore.isIngesting
  )
})

const canIngest = computed(() => {
  return mode.value === 'pdf' ? canIngestPdf.value : canIngestImages.value
})

// Reset form when dialog closes
watch(
  () => uiStore.isIngestDialogOpen,
  (isOpen) => {
    if (!isOpen) {
      mode.value = 'pdf'
      selectedFile.value = null
      selectedFiles.value = []
      titleOverride.value = ''
      authorOverride.value = ''
      documentTitle.value = ''
      ingestStore.reset()
    }
  }
)

async function handleSelectFile() {
  const result = await open({
    multiple: false,
    filters: [
      {
        name: 'PDF Documents',
        extensions: ['pdf'],
      },
    ],
  })

  if (result) {
    selectedFile.value = result as string
  }
}

async function handleSelectImages() {
  const result = await open({
    multiple: true,
    filters: [
      {
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'webp'],
      },
    ],
  })

  if (result) {
    selectedFiles.value = result as string[]
  }
}

function removeImage(index: number) {
  selectedFiles.value = selectedFiles.value.filter((_, i) => i !== index)
}

async function handleIngest() {
  if (mode.value === 'pdf') {
    if (!selectedFile.value) return

    const result = await ingestStore.ingestPdf(
      selectedFile.value,
      titleOverride.value || undefined,
      authorOverride.value || undefined
    )

    if (result) {
      console.log('Ingestion successful:', result)
      await documentsStore.loadDocuments()
      console.log('Files loaded:', documentsStore.documents)
    }
  } else {
    if (selectedFiles.value.length === 0 || !documentTitle.value.trim()) return

    const result = await ingestStore.ingestImages(
      selectedFiles.value,
      documentTitle.value.trim()
    )

    if (result) {
      console.log('Image ingestion successful:', result)
      await documentsStore.loadDocuments()
      console.log('Files loaded:', documentsStore.documents)
    }
  }
}

function handleClose() {
  uiStore.closeIngestDialog()
}
</script>

<template>
  <Dialog v-model:open="uiStore.isIngestDialogOpen">
    <DialogContent class="bg-gray-800 border-gray-700 text-gray-100 sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Import Document</DialogTitle>
        <DialogDescription class="text-gray-400">
          Import a PDF file or multiple images into the database.
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-4">
        <!-- Not Connected Warning -->
        <div
          v-if="!connectionStore.isConnected"
          class="rounded-md bg-yellow-900/50 p-3 text-sm text-yellow-300"
        >
          <span class="i-mdi-alert mr-2" />
          Please connect to a database first.
        </div>

        <!-- Mode Toggle -->
        <div class="flex rounded-lg bg-gray-700/50 p-1">
          <button
            type="button"
            class="flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="mode === 'pdf' ? 'bg-gray-600 text-white' : 'text-gray-400 hover:text-gray-200'"
            :disabled="ingestStore.isIngesting"
            @click="mode = 'pdf'"
          >
            <span class="i-mdi-file-pdf-box mr-1.5" />
            PDF
          </button>
          <button
            type="button"
            class="flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="mode === 'images' ? 'bg-gray-600 text-white' : 'text-gray-400 hover:text-gray-200'"
            :disabled="ingestStore.isIngesting"
            @click="mode = 'images'"
          >
            <span class="i-mdi-image-multiple mr-1.5" />
            Images
          </button>
        </div>

        <!-- PDF Mode -->
        <template v-if="mode === 'pdf'">
          <!-- File Selection -->
          <div class="space-y-2">
            <Label>PDF File</Label>
            <div class="flex gap-2">
              <Input
                :model-value="fileName"
                readonly
                class="flex-1 bg-gray-700 border-gray-600"
                placeholder="No file selected"
              />
              <Button
                type="button"
                variant="outline"
                class="border-gray-600 hover:bg-gray-700"
                :disabled="ingestStore.isIngesting"
                @click="handleSelectFile"
              >
                <span class="i-mdi-folder-open mr-2" />
                Browse
              </Button>
            </div>
          </div>

          <!-- Optional Overrides -->
          <div class="space-y-4">
            <div class="space-y-2">
              <Label for="title-override">Title (optional)</Label>
              <Input
                id="title-override"
                v-model="titleOverride"
                class="bg-gray-700 border-gray-600"
                placeholder="Override PDF title"
                :disabled="ingestStore.isIngesting"
              />
            </div>

            <div class="space-y-2">
              <Label for="author-override">Author (optional)</Label>
              <Input
                id="author-override"
                v-model="authorOverride"
                class="bg-gray-700 border-gray-600"
                placeholder="Override PDF author"
                :disabled="ingestStore.isIngesting"
              />
            </div>
          </div>
        </template>

        <!-- Images Mode -->
        <template v-else>
          <!-- Document Title (required) -->
          <div class="space-y-2">
            <Label for="document-title">Document Title <span class="text-red-400">*</span></Label>
            <Input
              id="document-title"
              v-model="documentTitle"
              class="bg-gray-700 border-gray-600"
              placeholder="Enter document title"
              :disabled="ingestStore.isIngesting"
            />
          </div>

          <!-- Image Selection -->
          <div class="space-y-2">
            <Label>Image Files</Label>
            <Button
              type="button"
              variant="outline"
              class="w-full border-gray-600 hover:bg-gray-700"
              :disabled="ingestStore.isIngesting"
              @click="handleSelectImages"
            >
              <span class="i-mdi-image-plus mr-2" />
              Select Images
            </Button>
          </div>

          <!-- Selected Files List -->
          <div v-if="selectedFiles.length > 0" class="space-y-2">
            <Label class="text-gray-400">{{ selectedFiles.length }} file(s) selected</Label>
            <div class="max-h-32 overflow-y-auto rounded-md bg-gray-700/50 p-2 space-y-1">
              <div
                v-for="(name, index) in selectedFileNames"
                :key="index"
                class="flex items-center justify-between rounded px-2 py-1 text-sm hover:bg-gray-600/50"
              >
                <span class="truncate flex-1">{{ name }}</span>
                <button
                  type="button"
                  class="ml-2 text-gray-400 hover:text-red-400"
                  :disabled="ingestStore.isIngesting"
                  @click="removeImage(index)"
                >
                  <span class="i-mdi-close" />
                </button>
              </div>
            </div>
          </div>
        </template>

        <!-- Progress -->
        <div v-if="ingestStore.progress" class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span class="text-gray-400">{{ ingestStore.progress.message }}</span>
            <span class="text-gray-400">{{ ingestStore.progressPercent }}%</span>
          </div>
          <Progress :model-value="ingestStore.progressPercent" class="h-2" />
        </div>

        <!-- Error Message -->
        <div
          v-if="ingestStore.error && !ingestStore.isIngesting"
          class="rounded-md bg-red-900/50 p-3 text-sm text-red-300"
        >
          <span class="i-mdi-alert-circle mr-2" />
          {{ ingestStore.error }}
        </div>

        <!-- Success Message -->
        <div
          v-if="ingestStore.isComplete && ingestStore.lastResult"
          class="rounded-md bg-green-900/50 p-3 text-sm text-green-300"
        >
          <span class="i-mdi-check-circle mr-2" />
          Successfully imported {{ ingestStore.lastResult.page_count }} {{ mode === 'pdf' ? 'pages' : 'images' }}.
        </div>
      </div>

      <DialogFooter class="gap-2">
        <Button
          type="button"
          variant="outline"
          class="border-gray-600 hover:bg-gray-700"
          @click="handleClose"
        >
          {{ ingestStore.isComplete ? 'Close' : 'Cancel' }}
        </Button>
        <Button
          v-if="!ingestStore.isComplete"
          type="button"
          :disabled="!canIngest"
          class="bg-blue-600 hover:bg-blue-700"
          @click="handleIngest"
        >
          <span v-if="ingestStore.isIngesting" class="i-mdi-loading animate-spin mr-2" />
          <span v-else class="i-mdi-file-import mr-2" />
          {{ ingestStore.isIngesting ? 'Importing...' : 'Import' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
