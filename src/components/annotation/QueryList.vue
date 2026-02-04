<script setup lang="ts">
import { onMounted } from 'vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useAnnotationStore, useConnectionStore } from '@/stores'

const annotationStore = useAnnotationStore()
const connectionStore = useConnectionStore()

onMounted(async () => {
  if (connectionStore.isConnected) {
    await annotationStore.loadQueries()
  }
})

async function handleEdit(queryId: number) {
  await annotationStore.startEditingQuery(queryId)
}

async function handleDelete(queryId: number) {
  if (confirm('Are you sure you want to delete this query?')) {
    await annotationStore.deleteQuery(queryId)
  }
}

function formatAnswers(answers: string[] | null): string {
  if (!answers || answers.length === 0) return ''
  const firstAnswer = answers[0]
  if (firstAnswer === undefined) return ''
  if (answers.length === 1) return firstAnswer
  return `${firstAnswer} (+${answers.length - 1} more)`
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-gray-200">
        Saved Queries
        <span v-if="annotationStore.queries.length > 0" class="text-gray-500">
          ({{ annotationStore.queries.length }})
        </span>
      </h3>
      <Button
        variant="ghost"
        size="sm"
        class="h-7 w-7 p-0"
        :disabled="!connectionStore.isConnected"
        @click="annotationStore.loadQueries()"
      >
        <span
          class="i-mdi-refresh text-lg"
          :class="annotationStore.isLoading && 'animate-spin'"
        />
      </Button>
    </div>

    <!-- Loading State -->
    <div v-if="annotationStore.isLoading" class="space-y-3">
      <Skeleton v-for="i in 3" :key="i" class="h-24 w-full bg-gray-700" />
    </div>

    <!-- Empty State -->
    <div
      v-else-if="annotationStore.queries.length === 0"
      class="flex flex-col items-center justify-center p-8 text-center text-gray-500"
    >
      <span class="i-mdi-comment-question-outline text-4xl mb-4" />
      <p class="text-sm">No queries created yet</p>
      <p class="text-xs mt-1">Create your first query in the Create tab</p>
    </div>

    <!-- Query List -->
    <div v-else class="space-y-3">
      <Card
        v-for="query in annotationStore.queries"
        :key="query.id"
        class="bg-gray-700/50 border-gray-600"
      >
        <CardHeader class="pb-2">
          <div class="flex items-start justify-between gap-2">
            <CardTitle class="text-sm font-medium text-gray-200 line-clamp-2">
              {{ query.contents }}
            </CardTitle>
            <div class="flex items-center gap-1 flex-shrink-0">
              <Button
                variant="ghost"
                size="sm"
                class="h-7 w-7 p-0 text-gray-400 hover:text-blue-400"
                @click="handleEdit(query.id)"
              >
                <span class="i-mdi-pencil" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-7 w-7 p-0 text-gray-400 hover:text-red-400"
                @click="handleDelete(query.id)"
              >
                <span class="i-mdi-delete" />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent class="pt-0">
          <div v-if="query.generation_gt && query.generation_gt.length > 0" class="text-xs text-gray-400 line-clamp-2 mb-2">
            {{ formatAnswers(query.generation_gt) }}
          </div>
          <div class="text-xs text-gray-500">
            ID: {{ query.id }}
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
