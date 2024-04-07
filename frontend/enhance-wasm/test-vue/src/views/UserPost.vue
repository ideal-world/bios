<script setup lang="ts">
import http from '../request/http'
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
const { params } = useRoute()

interface Post {
  id: string
  title: string
  body: string
}

const post = ref<Post>({
  id: '',
  title: '',
  body: ''
} as Post)

onMounted(async () => {
  // const response = await http.get('https://jsonplaceholder.typicode.com/posts/' + params.postId + "?q=测试")
  // post.value = response.data
  const response = await http.post('https://jsonplaceholder.typicode.com/posts', {
    userId: params.username,
    title: params.postId + '测试',
    body: 'Article body content'
  })
  post.value = response.data
})

</script>

<template>
  <h2>
    User {{ $route.params.username }} with post {{ $route.params.postId }}
  </h2>
  <h3>{{ post.id }} {{ post.title }}</h3>
  <p>{{ post.body }}</p>
</template>