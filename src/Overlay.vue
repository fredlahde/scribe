<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type OverlayMode = "warmup" | "waveform" | "spinner";

const mode = ref<OverlayMode>("waveform");
const bars = Array.from({ length: 16 }, (_, i) => i);

let unlistenMode: UnlistenFn | null = null;
let unlistenLevel: UnlistenFn | null = null;
let animationFrame: number | null = null;

// Animation constants
const WAVE_SPEED = 120; // ms per wave cycle (faster)
const PHASE_OFFSET = 0.4; // phase offset between bars
const INTERPOLATION_FACTOR = 0.5; // faster response to audio

let currentLevel = 0;
let targetLevel = 0;

onMounted(async () => {
  unlistenMode = await listen<OverlayMode>("overlay-mode", (event) => {
    mode.value = event.payload;
  });

  unlistenLevel = await listen<number>("audio-level", (event) => {
    // Amplify the incoming audio level
    targetLevel = Math.min(1, event.payload * 4);
  });

  // Start animation loop
  animate();
});

onUnmounted(() => {
  unlistenMode?.();
  unlistenLevel?.();
  if (animationFrame !== null) {
    cancelAnimationFrame(animationFrame);
  }
});

function animate() {
  // Warmup mode - gentle pulsing wave animation
  if (mode.value === "warmup") {
    const time = Date.now();
    bars.forEach((_, index) => {
      // Create a wave that sweeps across the bars
      const waveSpeed = 1000; // ms for full cycle (slower)
      const phase = ((time / waveSpeed) + (index * 0.12)) * Math.PI * 2;
      const wave = (Math.sin(phase) + 1) / 2; // 0 to 1

      // Height ranges from 8px to 18px (less dramatic)
      const height = 8 + wave * 10;
      // Opacity ranges from 0.6 to 1.0 (more subtle)
      const opacity = 0.6 + wave * 0.4;

      const barElement = document.querySelector(`.waveform-bar:nth-child(${index + 1})`) as HTMLElement;
      if (barElement) {
        barElement.style.height = height + "px";
        barElement.style.opacity = String(opacity);
      }
    });

    animationFrame = requestAnimationFrame(animate);
    return;
  }

  // Reset opacity for normal mode
  bars.forEach((_, index) => {
    const barElement = document.querySelector(`.waveform-bar:nth-child(${index + 1})`) as HTMLElement;
    if (barElement) {
      barElement.style.opacity = "";
    }
  });

  // Smooth interpolation
  currentLevel += (targetLevel - currentLevel) * INTERPOLATION_FACTOR;

  bars.forEach((_, index) => {
    // Create wave pattern with more variation
    const phase = (Date.now() / WAVE_SPEED + index * PHASE_OFFSET) % (Math.PI * 2);
    const wave = Math.sin(phase); // -1 to 1

    // Secondary wave for more organic movement
    const phase2 = (Date.now() / (WAVE_SPEED * 1.7) + index * PHASE_OFFSET * 0.7) % (Math.PI * 2);
    const wave2 = Math.sin(phase2) * 0.3;

    // Combined wave effect (0.2 to 1.0 range for more dramatic variation)
    const combinedWave = (wave + wave2) * 0.5 + 0.5; // normalize to 0-1
    const waveMultiplier = 0.2 + combinedWave * 0.8;

    // Stronger amplitude: base height + (level * max_amplitude * wave_variation)
    const height = 3 + (currentLevel * 25 * waveMultiplier);
    const barElement = document.querySelector(`.waveform-bar:nth-child(${index + 1})`) as HTMLElement;
    if (barElement) {
      barElement.style.height = Math.max(3, Math.min(28, height)) + "px";
    }
  });

  animationFrame = requestAnimationFrame(animate);
}
</script>

<template>
  <div class="overlay-container" :class="{ 'with-text': mode === 'warmup' }">
    <div v-if="mode === 'warmup' || mode === 'waveform'" class="waveform" :class="{ warmup: mode === 'warmup' }">
      <div v-for="i in bars" :key="i" class="waveform-bar"></div>
    </div>
    <div v-if="mode === 'spinner'" class="spinner">
      <div v-for="i in 8" :key="i" class="spinner-blade" :style="spinnerBladeStyle(i - 1)"></div>
    </div>
    <span v-if="mode === 'warmup'" class="warmup-text">Starting up</span>
  </div>
</template>

<script lang="ts">
export default {
  methods: {
    spinnerBladeStyle(index: number) {
      const bladeCount = 8;
      const rotation = (360 / bladeCount) * index;
      const delay = (1 / bladeCount) * index;
      return {
        transform: `rotate(${rotation}deg)`,
        animationDelay: `${-delay}s`,
      };
    },
  },
};
</script>
