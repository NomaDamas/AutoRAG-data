<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useDocumentsStore, useSelectionStore, useUiStore, type PageWithChunks } from '@/stores'

const documentsStore = useDocumentsStore()
const selectionStore = useSelectionStore()
const uiStore = useUiStore()

function getPageInfo(pageWithChunks: PageWithChunks) {
  return {
    ...pageWithChunks,
    thumbnailUrl: documentsStore.getThumbnailUrl(pageWithChunks.page.id),
    score: pageWithChunks.chunks[0] ? selectionStore.getChunkScore(pageWithChunks.chunks[0].id) : 1,
  }
}

function handlePreview(pageId: number) {
  uiStore.openPreview(pageId)
}

function handleRemove(pageId: number) {
  selectionStore.removeFromSelection(pageId)
}

function handleSetScore(pageWithChunks: PageWithChunks, score: number) {
  for (const chunk of pageWithChunks.chunks) {
    selectionStore.setChunkScore(chunk.id, score)
  }
}

const documentTitle = computed(() => {
  const doc = documentsStore.currentDocumentInfo
  if (!doc) return 'Untitled'
  return doc.title || doc.filename || 'Untitled'
})

const scoreOptions = [
  { val: 0, label: '0', title: 'Not relevant (hard negative)' },
  { val: 1, label: '1', title: 'Somewhat relevant (default)' },
  { val: 2, label: '2', title: 'Highly relevant' },
]

const showModeToggle = computed(() => selectionStore.selectedCount >= 2)

const groupCount = computed(() => selectionStore.groupedPages.length)

function handleGroupChange(pageId: number, event: Event) {
  const value = (event.target as HTMLSelectElement).value
  if (value === 'new') {
    selectionStore.splitToNewGroup(pageId)
  } else {
    selectionStore.mergeIntoGroup(Number(value), pageId)
  }
}

function findCurrentGroupIndex(pageId: number): number {
  const groups = selectionStore.groupedPages
  for (const grp of groups) {
    if (grp.pages.some((pg) => pg.page.id === pageId)) {
      return grp.groupIndex
    }
  }
  return 0
}
</script>

<template>
  <div class="space-y-2">
    <!-- Empty State -->
    <div
      v-if="!selectionStore.hasSelection"
      class="flex flex-col items-center justify-center rounded-lg border border-dashed border-gray-600 p-6 text-center"
    >
      <span class="i-mdi-image-multiple-outline text-3xl text-gray-500 mb-2" />
      <p class="text-sm text-gray-500">
        Click pages in the grid to add them as evidence
      </p>
      <p class="text-xs text-gray-600 mt-1">
        Use Cmd+click for multi-select, Shift+click for range
      </p>
    </div>

    <!-- Mode Toggle -->
    <div
      v-if="selectionStore.hasSelection && showModeToggle"
      class="flex items-center gap-1 text-xs"
    >
      <span class="text-gray-500 mr-1">Grouping:</span>
      <button
        class="rounded px-2 py-0.5 transition-colors"
        :class="selectionStore.groupingMode === 'and_all'
          ? 'bg-amber-600/80 text-white'
          : 'bg-gray-700 text-gray-400 hover:text-gray-300'"
        @click="selectionStore.setGroupingMode('and_all')"
      >
        AND all
      </button>
      <button
        class="rounded px-2 py-0.5 transition-colors"
        :class="selectionStore.groupingMode === 'custom'
          ? 'bg-amber-600/80 text-white'
          : 'bg-gray-700 text-gray-400 hover:text-gray-300'"
        @click="selectionStore.setGroupingMode('custom')"
      >
        Custom
      </button>
    </div>

    <!-- Evidence Items: AND all mode (flat list) -->
    <div v-if="selectionStore.hasSelection && selectionStore.groupingMode === 'and_all'" class="space-y-2">
      <div
        v-for="item in selectionStore.selectedPages"
        :key="item.page.id"
        class="flex items-center gap-3 rounded-lg bg-gray-700/50 p-2"
      >
        <!-- Thumbnail -->
        <button
          class="h-12 w-10 flex-shrink-0 overflow-hidden rounded bg-gray-700"
          @click="handlePreview(item.page.id)"
        >
          <img
            v-if="getPageInfo(item).thumbnailUrl"
            :src="getPageInfo(item).thumbnailUrl"
            :alt="`Page ${item.page.page_num}`"
            class="h-full w-full object-cover"
          />
          <div v-else class="flex h-full w-full items-center justify-center">
            <span class="i-mdi-file-document-outline text-gray-500" />
          </div>
        </button>

        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-200">
              Page {{ item.page.page_num }}
            </span>
            <Badge
              v-if="item.chunks.length > 0"
              variant="secondary"
              class="text-xs bg-gray-600"
            >
              {{ item.chunks.length }} chunk{{ item.chunks.length > 1 ? 's' : '' }}
            </Badge>
          </div>
          <p class="text-xs text-gray-500 truncate">
            {{ documentTitle }}
          </p>
        </div>

        <!-- Score Selector -->
        <div class="flex gap-1 items-center">
          <Button
            v-for="s in scoreOptions"
            :key="s.val"
            :variant="getPageInfo(item).score === s.val ? (s.val === 0 ? 'destructive' : s.val === 2 ? 'default' : 'secondary') : 'outline'"
            size="sm"
            class="h-6 w-6 p-0"
            :title="s.title"
            @click="handleSetScore(item, s.val)"
          >
            {{ s.label }}
          </Button>
        </div>

        <!-- Actions -->
        <Button
          variant="ghost"
          size="sm"
          class="h-8 w-8 p-0 text-gray-400 hover:text-red-400"
          @click="handleRemove(item.page.id)"
        >
          <span class="i-mdi-close" />
        </Button>
      </div>
    </div>

    <!-- Evidence Items: Custom mode (grouped) -->
    <div v-if="selectionStore.hasSelection && selectionStore.groupingMode === 'custom'" class="space-y-0">
      <template v-for="(group, gIdx) in selectionStore.groupedPages" :key="group.groupIndex">
        <!-- AND separator between groups -->
        <div v-if="gIdx > 0" class="flex items-center gap-2 py-2">
          <div class="flex-1 border-t border-gray-600/50" />
          <span class="text-xs font-medium text-amber-500 bg-amber-500/10 rounded px-2 py-0.5">AND</span>
          <div class="flex-1 border-t border-gray-600/50" />
        </div>

        <!-- Group container -->
        <div class="rounded-lg border border-gray-600/50 bg-gray-800/30 p-2 space-y-1">
          <!-- Group header -->
          <div class="flex items-center gap-2 text-xs text-gray-400 pb-1">
            <span class="font-medium">Group {{ gIdx + 1 }}</span>
            <Badge
              v-if="group.pages.length > 1"
              variant="secondary"
              class="text-[10px] bg-amber-600/20 text-amber-400 px-1.5 py-0"
            >
              OR
            </Badge>
          </div>

          <!-- Items in group -->
          <div
            v-for="item in group.pages"
            :key="item.page.id"
            class="flex items-center gap-3 rounded bg-gray-700/50 p-2"
          >
            <!-- Thumbnail -->
            <button
              class="h-12 w-10 flex-shrink-0 overflow-hidden rounded bg-gray-700"
              @click="handlePreview(item.page.id)"
            >
              <img
                v-if="getPageInfo(item).thumbnailUrl"
                :src="getPageInfo(item).thumbnailUrl"
                :alt="`Page ${item.page.page_num}`"
                class="h-full w-full object-cover"
              />
              <div v-else class="flex h-full w-full items-center justify-center">
                <span class="i-mdi-file-document-outline text-gray-500" />
              </div>
            </button>

            <!-- Info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-gray-200">
                  Page {{ item.page.page_num }}
                </span>
                <Badge
                  v-if="item.chunks.length > 0"
                  variant="secondary"
                  class="text-xs bg-gray-600"
                >
                  {{ item.chunks.length }} chunk{{ item.chunks.length > 1 ? 's' : '' }}
                </Badge>
              </div>
              <p class="text-xs text-gray-500 truncate">
                {{ documentTitle }}
              </p>
            </div>

            <!-- Score Selector -->
            <div class="flex gap-1 items-center">
              <Button
                v-for="s in scoreOptions"
                :key="s.val"
                :variant="getPageInfo(item).score === s.val ? (s.val === 0 ? 'destructive' : s.val === 2 ? 'default' : 'secondary') : 'outline'"
                size="sm"
                class="h-6 w-6 p-0"
                :title="s.title"
                @click="handleSetScore(item, s.val)"
              >
                {{ s.label }}
              </Button>
            </div>

            <!-- Group move dropdown -->
            <select
              class="h-7 rounded bg-gray-700 border border-gray-600 text-xs text-gray-300 px-1 cursor-pointer"
              :value="findCurrentGroupIndex(item.page.id)"
              title="Move to group"
              @change="handleGroupChange(item.page.id, $event)"
            >
              <option
                v-for="(g, gi) in selectionStore.groupedPages"
                :key="g.groupIndex"
                :value="g.groupIndex"
              >
                G{{ gi + 1 }}
              </option>
              <option value="new">+ New</option>
            </select>

            <!-- Remove -->
            <Button
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 text-gray-400 hover:text-red-400"
              @click="handleRemove(item.page.id)"
            >
              <span class="i-mdi-close" />
            </Button>
          </div>
        </div>
      </template>
    </div>

    <!-- Summary -->
    <div
      v-if="selectionStore.hasSelection"
      class="flex items-center justify-between text-xs text-gray-500 pt-2"
    >
      <span v-if="selectionStore.groupingMode === 'and_all' || groupCount <= 1">
        {{ selectionStore.selectedCount }} page{{ selectionStore.selectedCount > 1 ? 's' : '' }} selected
      </span>
      <span v-else>
        {{ selectionStore.selectedCount }} page{{ selectionStore.selectedCount > 1 ? 's' : '' }} in {{ groupCount }} group{{ groupCount > 1 ? 's' : '' }}
      </span>
      <button
        class="text-gray-400 hover:text-gray-300"
        @click="selectionStore.clearSelection()"
      >
        Clear all
      </button>
    </div>
  </div>
</template>
