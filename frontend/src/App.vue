<template>
  <NameModal v-if="!santaStore.currentUser" />
  <RouterView v-else />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { useSantaStore } from '@/stores/santa'
import NameModal from './components/NameModal.vue'

const santaStore = useSantaStore()

onMounted(() => {
  santaStore.getSiteUser()
})
</script>
