<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import p5 from 'p5';

  let canvasContainer: HTMLDivElement;
  let sketchInstance: p5 | null = null;

  // p5.js sketch
	function sketch(p: p5) {
		let time = 0;
		let noiseScale = 0.005;
		let timeScale = 0.003; // Reduced to 20% of original speed (0.01 * 0.2)
		let contourSpacing = 1.6; // Distance between contour lines (increased for bigger gaps)
		let heightScale = 2; // Amplitude of the heightmap
		let resolution = 10; // Grid resolution for performance (increased to reduce computation)

		p.setup = () => {
			p.createCanvas(p.windowWidth, p.windowHeight);
			p.background(0);
			p.stroke(255);
			p.strokeWeight(0.8);
			p.noFill();
		};

		p.draw = () => {
			p.background(0, 10); // Slight fade for animation trail effect
			
			time += timeScale;
			
			// Draw contour lines
			for (let level = 0; level <= 1; level++) {
				let targetHeight = level * contourSpacing;
				drawContourLine(targetHeight, level);
			}
		};

		function drawContourLine(targetHeight: number, level: number) {
			p.stroke(255, (60 + level * 15) * 0.1); // Varying opacity for depth, dimmed by 50%
			
			// Scan through the canvas and find points at the target height
			for (let x = 0; x < p.width; x += resolution) {
				for (let y = 0; y < p.height; y += resolution) {
					let height1 = getHeight(x, y);
					let height2 = getHeight(x + resolution, y);
					let height3 = getHeight(x, y + resolution);
					let height4 = getHeight(x + resolution, y + resolution);
					
					// Check if contour line passes through this cell
					if (isContourCrossing(height1, height2, height3, height4, targetHeight)) {
						// Draw interpolated contour segments
						drawContourSegments(x, y, resolution, targetHeight);
					}
				}
			}
		}

		function getHeight(x: number, y: number): number {
			// Create layered noise for more interesting terrain
			let height = 0;
			height += p.noise(x * noiseScale, y * noiseScale, time) * heightScale;
			height += p.noise(x * noiseScale * 2, y * noiseScale * 2, time * 1.5) * heightScale * 0.5;
			height += p.noise(x * noiseScale * 4, y * noiseScale * 4, time * 2) * heightScale * 0.25;
			return height;
		}

		function isContourCrossing(h1: number, h2: number, h3: number, h4: number, target: number): boolean {
			let min = Math.min(h1, h2, h3, h4);
			let max = Math.max(h1, h2, h3, h4);
			return min <= target && target <= max;
		}

		function drawContourSegments(x: number, y: number, res: number, target: number) {
			let h1 = getHeight(x, y);
			let h2 = getHeight(x + res, y);
			let h3 = getHeight(x + res, y + res);
			let h4 = getHeight(x, y + res);
			
			let points: Array<{x: number, y: number}> = [];
			
			// Check each edge of the cell for intersection
			if ((h1 <= target && h2 >= target) || (h1 >= target && h2 <= target)) {
				let t = (target - h1) / (h2 - h1);
				points.push({ x: x + t * res, y: y });
			}
			
			if ((h2 <= target && h3 >= target) || (h2 >= target && h3 <= target)) {
				let t = (target - h2) / (h3 - h2);
				points.push({ x: x + res, y: y + t * res });
			}
			
			if ((h3 <= target && h4 >= target) || (h3 >= target && h4 <= target)) {
				let t = (target - h3) / (h4 - h3);
				points.push({ x: x + (1 - t) * res, y: y + res });
			}
			
			if ((h4 <= target && h1 >= target) || (h4 >= target && h1 <= target)) {
				let t = (target - h4) / (h1 - h4);
				points.push({ x: x, y: y + (1 - t) * res });
			}
			
			// Draw line segments between intersection points
			if (points.length >= 2) {
				for (let i = 0; i < points.length - 1; i += 2) {
					if (points[i + 1]) {
						p.line(points[i].x, points[i].y, points[i + 1].x, points[i + 1].y);
					}
				}
			}
		}

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
