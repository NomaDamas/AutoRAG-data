<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { useAnnotationStore, useSelectionStore } from '@/stores'

const annotationStore = useAnnotationStore()
const selectionStore = useSelectionStore()

const saveSuccess = ref(false)

async function handleSave() {
  if (annotationStore.editingQuery) {
    const result = await annotationStore.updateQuery()
    if (result) {
      showSaveSuccess()
      selectionStore.clearEvidence()
    }
  } else {
    const result = await annotationStore.saveQuery()
    if (result) {
      showSaveSuccess()
      selectionStore.clearEvidence()
    }
  }
}

function showSaveSuccess() {
  saveSuccess.value = true
  setTimeout(() => {
    saveSuccess.value = false
  }, 2000)
}

function handleCancel() {
  annotationStore.cancelEditing()
}

</script>

<template>
  <form class="space-y-4" @submit.prevent="handleSave">
    <!-- Contents (Question) -->
    <div class="space-y-2">
      <Label for="contents" class="text-gray-300">Question / Query</Label>
      <Textarea
        id="contents"
        v-model="annotationStore.draftContents"
        placeholder="Enter the question that this evidence answers..."
        class="min-h-[80px] bg-gray-700 border-gray-600 text-gray-100 placeholder:text-gray-500"
      />
    </div>

    <!-- Query to LLM -->
    <div class="space-y-2">
      <Label for="queryToLlm" class="text-gray-300">
        Query to LLM
        <span class="text-gray-500 font-normal">(optional)</span>
      </Label>
      <Textarea
        id="queryToLlm"
        v-model="annotationStore.draftQueryToLlm"
        placeholder="Reformulated query for the LLM..."
        class="min-h-[60px] bg-gray-700 border-gray-600 text-gray-100 placeholder:text-gray-500"
      />
    </div>

    <!-- Generation GT - Multiple Answers -->
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <Label class="text-gray-300">
          Expected Answers
          <span class="text-gray-500 font-normal">(optional)</span>
        </Label>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-7 text-xs text-blue-400 hover:text-blue-300"
          @click="annotationStore.addGenerationGt()"
        >
          <span class="i-mdi-plus mr-1" />
          Add Answer
        </Button>
      </div>
      <div class="space-y-2">
        <div
          v-for="(answer, index) in annotationStore.draftGenerationGt"
          :key="index"
          class="flex items-start gap-2"
        >
          <Textarea
            v-model="annotationStore.draftGenerationGt[index]"
            :placeholder="`Answer ${index + 1}...`"
            class="flex-1 min-h-[60px] bg-gray-700 border-gray-600 text-gray-100 placeholder:text-gray-500"
          />
          <Button
            v-if="annotationStore.draftGenerationGt.length > 1"
            type="button"
            variant="ghost"
            size="sm"
            class="h-8 w-8 p-0 text-gray-400 hover:text-red-400 flex-shrink-0"
            @click="annotationStore.removeGenerationGt(index)"
          >
            <span class="i-mdi-close" />
          </Button>
        </div>
      </div>
      <p class="text-xs text-gray-500">
        Multiple valid answers can be provided for queries with more than one correct response.
      </p>
    </div>

    <!-- Error Message -->
    <div
      v-if="annotationStore.error"
      class="rounded-md bg-red-900/50 p-3 text-sm text-red-300"
    >
      {{ annotationStore.error }}
    </div>

    <!-- Validation Warning -->
    <div
      v-if="!selectionStore.hasSelection && !annotationStore.editingQuery"
      class="rounded-md bg-yellow-900/50 p-3 text-sm text-yellow-300"
    >
      Select at least one page as evidence before saving.
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-2">
      <Button
        type="submit"
        :disabled="!annotationStore.canSave || annotationStore.isSaving || saveSuccess"
        :class="saveSuccess
          ? 'bg-green-600 hover:bg-green-600'
          : 'bg-blue-600 hover:bg-blue-700 disabled:opacity-50'"
      >
        <span v-if="annotationStore.isSaving" class="i-mdi-loading animate-spin mr-2" />
        <span v-else-if="saveSuccess" class="i-mdi-check mr-2" />
        <template v-if="saveSuccess">
          Saved!
        </template>
        <template v-else>
          {{ annotationStore.editingQuery ? 'Update' : 'Save' }}
        </template>
      </Button>
      <Button
        v-if="annotationStore.editingQuery"
        type="button"
        variant="outline"
        class="border-gray-600"
        @click="handleCancel"
      >
        Cancel
      </Button>
      <Button
        v-if="annotationStore.isDirty && !annotationStore.editingQuery"
        type="button"
        variant="ghost"
        class="text-gray-400"
        @click="annotationStore.clearDraft()"
      >
        Clear
      </Button>
    </div>
  </form>
</template>
