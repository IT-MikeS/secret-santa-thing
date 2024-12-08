<template>
  <div style="position: absolute; top: 0; left: 0;" class="h-screen">
    <Button
      icon="pi pi-angle-double-right"
      @click="showSidebar = !showSidebar"
      variant="text"
      style="z-index: 10;"
      class="h-full"
    />
  </div>
  <Drawer v-model:visible="showSidebar" style="z-index: 11;">
    <template #header>
      <h2 class="text-center m-0">Your Groups</h2>
    </template>
    <div class="flex flex-column gap-2">
      <Button
        v-for="group in santaStore.userGroups"
        :key="group.id"
        @click="router.push(`/group/${group.id}`)"
        style="border-color: #ff6b6b; border-radius: 5px;"
        variant="text"
      >
        <div class="flex flex-column">
          <div>{{ group.name }}</div>
          <div class="flex mt-2">
            <Tag severity="info" v-if="group.isCreator">Creator</Tag>
            <Tag class="ml-2"  severity="success" v-if="group.isGenerated">Generated</Tag>
          </div>
        </div>
      </Button>
    </div>
  </Drawer>
  <div class="w-screen" style="position: absolute; top: 0; left: 0; z-index: 9;">
    <Card class="p-0 mt-6" style="width: 300px; margin-left: calc(50% - 150px)">
      <template #title>
        <div class="text-center">
          <h2 class="mt-0">Secret Santa</h2>
        </div>
      </template>
      <template #content>
        <div class="flex flex-column gap-2">
          <Button
            label="Join Group"
            @click="showJoinModal = true"
            class="m-2 px-5 py-3"
          />
          <Button
            label="Create Group"
            @click="showCreateModal = true"
            severity="success"
            class="m-2 px-5 py-3"
          />
        </div>
      </template>
    </Card>
  </div>
  <Dialog header="Create Group" v-model:visible="showCreateModal" :modal="true" :closable="false">
    <form @submit.prevent="handleCreate" class="flex justify-center flex-column">
      <div>
        <InputText
          v-model="groupName"
          type="text"
          placeholder="Group Name"
          required
        />
      </div>
      <div class="flex flex-column gap-2 pt-4">
        <Button
          label="Create"
          type="submit"
          severity="success"
        />
        <Button
          label="Cancel"
          type="button"
          @click="showCreateModal = false"
        />
      </div>
    </form>
  </Dialog>

  <Dialog header="Join Group" v-model:visible="showJoinModal" :modal="true" :closable="false">
    <form @submit.prevent="handleJoin" class="flex justify-center flex-column">
      <div>
        <InputText
          v-model="joinCode"
          type="text"
          maxlength="5"
          required
          @input="e => joinCode = (e.target as HTMLInputElement).value.toUpperCase()"
          placeholder="Join Code (XXXXX)"
        />
      </div>
      <div class="flex flex-column gap-2 pt-4">
        <Button
          label="Join"
          type="submit"
          severity="success"
        />
        <Button
          label="Cancel"
          type="button"
          @click="showJoinModal = false"
        />
      </div>
    </form>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSantaStore } from '@/stores/santa'
import Drawer from 'primevue/drawer'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Card from 'primevue/card'

const router = useRouter()
const santaStore = useSantaStore()

const showCreateModal = ref(false)
const showJoinModal = ref(false)
const showSidebar = ref(false)
const groupName = ref('')
const joinCode = ref('')

onMounted(async () => {
  await santaStore.loadUserGroups()
})

async function handleCreate() {
  try {
    const groupId = await santaStore.createGroup(groupName.value)
    await santaStore.loadUserGroups()
    showCreateModal.value = false
    router.push(`/group/${groupId}`)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (error: any) {
    console.error(error.message)
  }
}

async function handleJoin() {
  try {
    await santaStore.joinGroup(joinCode.value)
    await santaStore.loadUserGroups()
    showJoinModal.value = false
    router.push(`/group/${joinCode.value}`)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (error: any) {
    console.error(error.message)
  }
}
</script>