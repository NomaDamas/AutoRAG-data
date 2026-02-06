<script setup lang="ts">
import { computed } from 'vue'
import type { PageInfo, ImageChunkInfo } from '@/stores/documents'

const props = defineProps<{
  page: PageInfo
  chunks: ImageChunkInfo[]
  isFocused: boolean
  isInEvidence: boolean
  showPageNumber: boolean
  thumbnailUrl?: string
}>()

const emit = defineEmits<{
  click: [event: MouseEvent]
  'toggle-evidence': []
}>()

const hasChunks = computed(() => props.chunks.length > 0)

function handleToggleEvidence(event: MouseEvent) {
  event.stopPropagation()
  emit('toggle-evidence')
}
</script>

<template>
  <button
    class="group relative aspect-[3/4] w-full overflow-hidden rounded-lg bg-gray-800 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500"
    :class="[
      isInEvidence
        ? 'ring-2 ring-amber-500 ring-offset-2 ring-offset-gray-900'
        : isFocused
          ? 'ring-2 ring-blue-500 ring-offset-2 ring-offset-gray-900'
          : 'hover:ring-2 hover:ring-gray-600',
    ]"
    @click="emit('click', $event)"
  >
    <!-- Thumbnail Image -->
    <img
      v-if="thumbnailUrl"
      :src="thumbnailUrl"
      :alt="`Page ${page.page_num}`"
      class="h-full w-full object-cover"
      loading="lazy"
    />

    <!-- Placeholder when no thumbnail -->
    <div v-else class="flex h-full w-full items-center justify-center bg-gray-700">
      <span class="i-mdi-file-document-outline text-4xl text-gray-500" />
    </div>

    <!-- Evidence Checkmark Badge (shown when in evidence) -->
    <button
      v-if="isInEvidence"
      class="absolute top-2 right-2 flex h-6 w-6 items-center justify-center rounded-full bg-amber-500 hover:bg-amber-400 transition-colors"
      title="Remove from evidence"
      @click="handleToggleEvidence"
    >
      <span class="i-mdi-check text-white" />
    </button>

    <!-- Add to Evidence Button (shown on hover when NOT in evidence) -->
    <button
      v-else
      class="absolute top-2 right-2 flex h-6 w-6 items-center justify-center rounded-full bg-gray-700/80 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-amber-500"
      title="Add to evidence"
      @click="handleToggleEvidence"
    >
      <span class="i-mdi-plus text-white" />
    </button>

    <!-- Chunk Badge -->
    <div
      v-if="hasChunks"
      class="absolute top-2 left-2 rounded bg-gray-800/80 px-1.5 py-0.5 text-xs text-gray-300"
    >
      {{ chunks.length }} chunk{{ chunks.length > 1 ? 's' : '' }}
    </div>

    <!-- Page Number -->
    <div
      v-if="showPageNumber"
      class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/70 to-transparent p-2"
    >
      <span class="text-sm font-medium text-white">{{ page.page_num }}</span>
    </div>

    <!-- Hover Overlay -->
    <div
      class="pointer-events-none absolute inset-0 bg-blue-500/0 transition-colors group-hover:bg-blue-500/10"
      :class="isInEvidence && 'group-hover:bg-amber-500/10'"
    />
  </button>
</template>
