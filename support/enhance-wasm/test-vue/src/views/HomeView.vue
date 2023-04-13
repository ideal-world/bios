<script setup lang="ts">
import * as bios from "bios-enhance-wasm"

import TheWelcome from '../components/TheWelcome.vue'

console.log('--------in views---------' + bios.encrypt("ddddd"));

</script>

<template>
  <main>
    <TheWelcome />
  </main>
</template>
