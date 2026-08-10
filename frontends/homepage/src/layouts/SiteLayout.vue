<template>
    <div class="site-shell">
        <AppHeader />
        <main class="site-main">
            <RouterView />
        </main>
        <AppFooter />
    </div>
</template>

<script setup lang="ts">
import AppHeader from "../components/AppHeader.vue";
import AppFooter from "../components/AppFooter.vue";
</script>
