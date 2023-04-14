<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <header>
    <nav>
      <RouterLink to="/">Home</RouterLink>
      <RouterLink to="/about">About</RouterLink>
      <RouterLink to="/user/gudaoxuri">/user/gudaoxuri</RouterLink>
      <RouterLink to="/user/gudaoxuri/profile">/user/gudaoxuri/profile</RouterLink>
      <RouterLink to="/users/eduardo/posts/1?name=张三">/users/eduardo/posts/1?name=张三</RouterLink>
      <RouterLink to="/users/gudaoxuri/posts/100">/users/gudaoxuri/posts/100</RouterLink>
    </nav>
  </header>

  <RouterView />
</template>

<style scoped>
nav {
  width: 100%;
  font-size: 20;
  text-align: center;
  margin-top: 2rem;
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
  border: 0;
}
</style>
