<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import p5 from 'p5';

  let canvasContainer: HTMLDivElement;
  let sketchInstance: p5 | null = null;

  // p5.js sketch
function sketch(p: p5) {
    type Dot = {
        x: number;
        y: number;
        size: number;
        speed: number;
        angle: number;
    };

    let dots: Array<Dot> = [];
    let dot_count = 100;


    p.setup = () => {
        p.createCanvas(p.windowWidth, p.windowHeight);
        p.noStroke();

        for (let i = 0; i < dot_count; i++) {
            dots.push({
                x: p.random(p.width),
                y: p.random(p.height),
                size: p.random(5, 15),
                speed: p.random(0.25, 1),
                angle: p.random(Math.PI * 2)
            });
        }
    };

    p.draw = () => {
        p.background(0, 5); // semi-transparent background for fading effect

        for (let dot of dots) {
            p.fill(105, 150); // white with some transparency
            p.ellipse(dot.x, dot.y, dot.size);
            
            // Update dot position
            dot.x += p.cos(dot.angle) * dot.speed;
            dot.y += p.sin(dot.angle) * dot.speed;

            // Wrap around edges
            if (dot.x < 0) dot.x = p.width;
            if (dot.x > p.width) dot.x = 0;
            if (dot.y < 0) dot.y = p.height;
            if (dot.y > p.height) dot.y = 0;

            // Randomly change direction
            dot.angle += p.random(Math.PI / 20, Math.PI / 10) * (p.random() < 0.5 ? 1 : -1);

            // Randomly change speed
            if (p.random() < 0.01) {
                dot.speed = p.random(0.5, 2);
            }
        }
    };

    p.windowResized = () => {
        p.resizeCanvas(p.windowWidth, p.windowHeight);
    };
}

  onMount(() => {
    sketchInstance = new p5(sketch, canvasContainer);
  });

  onDestroy(() => {
    if (sketchInstance) {
      sketchInstance.remove();
    }
  });
</script>

<style>
  :global(body) {
    margin: 0;
    overflow: hidden;
  }

  .background-canvas {
    position: fixed;
    top: 0;
    left: 0;
    z-index: -1; /* behind everything */
    width: 100vw;
    height: 100vh;
    pointer-events: none;
  }
</style>

<div bind:this={canvasContainer} class="background-canvas"></div>
