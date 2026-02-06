<script setup lang="ts">
import { ref } from 'vue'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { useConnectionStore, useSelectionStore, useAnnotationStore } from '@/stores'
import QueryForm from './QueryForm.vue'
import EvidenceList from './EvidenceList.vue'
import QueryList from './QueryList.vue'
import ExportPanel from '@/components/export/ExportPanel.vue'

const connectionStore = useConnectionStore()
const selectionStore = useSelectionStore()
const annotationStore = useAnnotationStore()

const activeTab = ref<'create' | 'list' | 'export'>('create')
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- Header with Tabs -->
    <div class="border-b border-gray-700">
      <div class="flex items-center px-4 py-2">
        <button
          class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors"
          :class="activeTab === 'create' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-gray-200'"
          @click="activeTab = 'create'"
        >
          Create
        </button>
        <button
          class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors ml-1"
          :class="activeTab === 'list' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-gray-200'"
          @click="activeTab = 'list'"
        >
          Queries
        </button>
        <button
          class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors ml-1"
          :class="activeTab === 'export' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-gray-200'"
          @click="activeTab = 'export'"
        >
          Export
        </button>
      </div>
    </div>

    <ScrollArea class="flex-1 min-h-0">
      <div class="p-4">
        <!-- Not Connected State -->
        <div
          v-if="!connectionStore.isConnected"
          class="flex flex-col items-center justify-center p-8 text-center text-gray-500"
        >
          <span class="i-mdi-pencil-off text-4xl mb-4" />
          <p class="text-sm">Connect to database to create annotations</p>
        </div>

        <!-- Create Tab -->
        <div v-else-if="activeTab === 'create'" class="space-y-6">
          <!-- Evidence Section -->
          <div>
            <h3 class="mb-3 text-sm font-medium text-gray-200">
              Evidence
              <span v-if="selectionStore.hasSelection" class="text-gray-500">
                ({{ selectionStore.selectedCount }} pages)
              </span>
            </h3>
            <EvidenceList />
          </div>

          <Separator class="bg-gray-700" />

          <!-- Query Form -->
          <div>
            <h3 class="mb-3 text-sm font-medium text-gray-200">
              {{ annotationStore.editingQuery ? 'Edit Query' : 'New Query' }}
            </h3>
            <QueryForm />
          </div>
        </div>

        <!-- List Tab -->
        <div v-else-if="activeTab === 'list'">
          <QueryList />
        </div>

        <!-- Export Tab -->
        <div v-else-if="activeTab === 'export'">
          <ExportPanel />
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
