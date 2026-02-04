<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Progress } from '@/components/ui/progress'
import { useExportStore, useConnectionStore } from '@/stores'

const exportStore = useExportStore()
const connectionStore = useConnectionStore()

const outputPath = ref('')
const createZip = ref(true)
const includeDocuments = ref(true)
const includeQueries = ref(true)
const includeRelations = ref(true)
const includeImageChunks = ref(true)
const includeImages = ref(true)

const canExport = computed(() => {
  return (
    connectionStore.isConnected &&
    outputPath.value &&
    !exportStore.isExporting &&
    (includeDocuments.value ||
      includeQueries.value ||
      includeRelations.value ||
      includeImageChunks.value ||
      includeImages.value)
  )
})

const totalItems = computed(() => {
  if (!exportStore.counts) return 0
  let total = 0
  if (includeDocuments.value) total += exportStore.counts.documents
  if (includeQueries.value) total += exportStore.counts.queries
  if (includeRelations.value) total += exportStore.counts.relations
  if (includeImageChunks.value || includeImages.value) total += exportStore.counts.image_chunks
  return total
})

onMounted(async () => {
  if (connectionStore.isConnected) {
    await exportStore.fetchCounts()
  }
})

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Export Folder',
  })

  if (selected && typeof selected === 'string') {
    outputPath.value = selected
  }
}

async function handleExport() {
  if (!canExport.value) return

  await exportStore.exportData({
    output_path: outputPath.value,
    create_zip: createZip.value,
    include_documents: includeDocuments.value,
    include_queries: includeQueries.value,
    include_relations: includeRelations.value,
    include_image_chunks: includeImageChunks.value,
    include_images: includeImages.value,
  })
}

function handleReset() {
  exportStore.reset()
  outputPath.value = ''
}
</script>

<template>
  <div class="space-y-6">
    <!-- Not Connected State -->
    <div
      v-if="!connectionStore.isConnected"
      class="flex flex-col items-center justify-center p-8 text-center text-gray-500"
    >
      <span class="i-mdi-export-variant text-4xl mb-4" />
      <p class="text-sm">Connect to database to export data</p>
    </div>

    <template v-else>
      <!-- Export Complete State -->
      <div v-if="exportStore.isComplete && exportStore.lastResult" class="space-y-4">
        <div class="rounded-md bg-green-900/50 p-4 text-green-300">
          <div class="flex items-center gap-2 mb-2">
            <span class="i-mdi-check-circle text-xl" />
            <span class="font-medium">Export Complete</span>
          </div>
          <p class="text-sm text-green-400 mb-3 break-all">
            {{ exportStore.lastResult.output_path }}
          </p>
          <div class="text-xs space-y-1">
            <p v-if="exportStore.lastResult.documents_count > 0">
              Documents: {{ exportStore.lastResult.documents_count }}
            </p>
            <p v-if="exportStore.lastResult.queries_count > 0">
              Queries: {{ exportStore.lastResult.queries_count }}
            </p>
            <p v-if="exportStore.lastResult.relations_count > 0">
              Relations: {{ exportStore.lastResult.relations_count }}
            </p>
            <p v-if="exportStore.lastResult.image_chunks_count > 0">
              Image Chunks: {{ exportStore.lastResult.image_chunks_count }}
            </p>
            <p v-if="exportStore.lastResult.images_count > 0">
              Images: {{ exportStore.lastResult.images_count }}
            </p>
          </div>
        </div>
        <Button class="w-full" variant="outline" @click="handleReset">
          Export Again
        </Button>
      </div>

      <!-- Export Form -->
      <template v-else>
        <!-- Output Folder Selection -->
        <div class="space-y-2">
          <Label class="text-gray-300">Output Folder</Label>
          <div class="flex gap-2">
            <div
              class="flex-1 bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-sm text-gray-300 truncate"
            >
              {{ outputPath || 'No folder selected' }}
            </div>
            <Button
              variant="outline"
              size="sm"
              class="border-gray-600 flex-shrink-0"
              :disabled="exportStore.isExporting"
              @click="selectFolder"
            >
              <span class="i-mdi-folder-open mr-1" />
              Browse
            </Button>
          </div>
        </div>

        <!-- ZIP Option -->
        <div class="flex items-center gap-2">
          <Checkbox
            id="createZip"
            :checked="createZip"
            :disabled="exportStore.isExporting"
            @update:checked="createZip = $event as boolean"
          />
          <Label for="createZip" class="text-gray-300 cursor-pointer">
            Compress as ZIP
          </Label>
        </div>

        <!-- Data Selection -->
        <div class="space-y-3">
          <Label class="text-gray-300">Data to Export</Label>

          <div class="space-y-2 pl-1">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Checkbox
                  id="includeDocuments"
                  :checked="includeDocuments"
                  :disabled="exportStore.isExporting"
                  @update:checked="includeDocuments = $event as boolean"
                />
                <Label for="includeDocuments" class="text-gray-300 cursor-pointer">
                  Documents
                </Label>
              </div>
              <span class="text-xs text-gray-500">
                {{ exportStore.counts?.documents ?? 0 }}
              </span>
            </div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Checkbox
                  id="includeQueries"
                  :checked="includeQueries"
                  :disabled="exportStore.isExporting"
                  @update:checked="includeQueries = $event as boolean"
                />
                <Label for="includeQueries" class="text-gray-300 cursor-pointer">
                  Queries
                </Label>
              </div>
              <span class="text-xs text-gray-500">
                {{ exportStore.counts?.queries ?? 0 }}
              </span>
            </div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Checkbox
                  id="includeRelations"
                  :checked="includeRelations"
                  :disabled="exportStore.isExporting"
                  @update:checked="includeRelations = $event as boolean"
                />
                <Label for="includeRelations" class="text-gray-300 cursor-pointer">
                  Retrieval Relations
                </Label>
              </div>
              <span class="text-xs text-gray-500">
                {{ exportStore.counts?.relations ?? 0 }}
              </span>
            </div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Checkbox
                  id="includeImageChunks"
                  :checked="includeImageChunks"
                  :disabled="exportStore.isExporting"
                  @update:checked="includeImageChunks = $event as boolean"
                />
                <Label for="includeImageChunks" class="text-gray-300 cursor-pointer">
                  Image Chunks (CSV)
                </Label>
              </div>
              <span class="text-xs text-gray-500">
                {{ exportStore.counts?.image_chunks ?? 0 }}
              </span>
            </div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Checkbox
                  id="includeImages"
                  :checked="includeImages"
                  :disabled="exportStore.isExporting"
                  @update:checked="includeImages = $event as boolean"
                />
                <Label for="includeImages" class="text-gray-300 cursor-pointer">
                  Image Files (PNG)
                </Label>
              </div>
              <span class="text-xs text-gray-500">
                {{ exportStore.counts?.image_chunks ?? 0 }}
              </span>
            </div>
          </div>
        </div>

        <!-- Progress -->
        <div v-if="exportStore.isExporting || exportStore.progress" class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span class="text-gray-400">{{ exportStore.progress?.message }}</span>
            <span v-if="exportStore.progress?.total" class="text-gray-500">
              {{ exportStore.progressPercent }}%
            </span>
          </div>
          <Progress :model-value="exportStore.progressPercent" class="h-2" />
        </div>

        <!-- Error -->
        <div
          v-if="exportStore.error && exportStore.isFailed"
          class="rounded-md bg-red-900/50 p-3 text-sm text-red-300"
        >
          {{ exportStore.error }}
        </div>

        <!-- Export Button -->
        <Button
          class="w-full"
          :disabled="!canExport"
          @click="handleExport"
        >
          <span v-if="exportStore.isExporting" class="i-mdi-loading animate-spin mr-2" />
          <span v-else class="i-mdi-export-variant mr-2" />
          {{ exportStore.isExporting ? 'Exporting...' : 'Export Data' }}
        </Button>

        <!-- Summary -->
        <p v-if="totalItems > 0" class="text-xs text-gray-500 text-center">
          {{ totalItems }} items will be exported
        </p>
      </template>
    </template>
  </div>
</template>
