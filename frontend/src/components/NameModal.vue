<template>
  <div class="flex align-content-center justify-content-center p-4">
    <div class="p-6" style="min-width: 450px;">
      <h2 class="text-2xl font-bold mb-4 text-center">Welcome to Secret Santa</h2>
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <InputText
            v-model="name"
            type="text"
            class="w-full"
            placeholder="Your Full Name"
            required
          />
          <p class="mt-1 pl-1 text-sm">This name will be used for all groups you join.</p>
        </div>
        <Button
          type="submit"
          class="w-full px-4 py-2"
          severity="success"
        >
          Continue
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useSantaStore } from '@/stores/santa'

const santaStore = useSantaStore()
const name = ref('')

function handleSubmit() {
  santaStore.setSiteUser(name.value)
}
</script>
