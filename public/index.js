document.addEventListener('DOMContentLoaded', function () {
	const videoInput = document.getElementById('video-input');
	const fileNameDisplay = document.getElementById('file-name');
	const conversionForm = document.getElementById('conversion-form');
	const videoPreview = document.getElementById('video-preview');

	const startSlider = document.getElementById('start-slider');
	const endSlider = document.getElementById('end-slider');
	const startTimeInput = document.getElementById('start_time');
	const endTimeInput = document.getElementById('end_time');
	const sliderHighlight = document.getElementById('slider-highlight');

	const resultSection = document.getElementById('result-section');
	const loader = document.getElementById('loader');
	const gifContainer = document.getElementById('gif-container');
	const errorContainer = document.getElementById('error-container');
	const gifPreview = document.getElementById('gif-preview');
	const downloadButton = document.getElementById('download-button');

	let videoFile = null;

	videoInput.addEventListener('change', (e) => {
		if (e.target.files && e.target.files[0]) {
			videoFile = e.target.files[0];
			fileNameDisplay.textContent = videoFile.name;
			const fileURL = URL.createObjectURL(videoFile);
			videoPreview.src = fileURL;
			conversionForm.classList.remove('hidden');
			resultSection.classList.add('hidden');
		}
	});

	videoPreview.addEventListener('loadedmetadata', () => {
		const duration = videoPreview.duration;
		startSlider.max = duration;
		endSlider.max = duration;
		startSlider.value = 0;
		endSlider.value = duration;
		startTimeInput.value = '0.0';
		endTimeInput.value = duration.toFixed(1);
		updateSliderHighlight();
	});

	function updateSliderHighlight() {
		const max = parseFloat(startSlider.max);
		const start = parseFloat(startSlider.value);
		const end = parseFloat(endSlider.value);

		sliderHighlight.style.left = `${(start / max) * 100}%`;
		sliderHighlight.style.width = `${((end - start) / max) * 100}%`;
	}

	function formatTime(seconds) {
		return parseFloat(seconds).toFixed(1);
	}

	startSlider.addEventListener('input', () => {
		if (parseFloat(startSlider.value) >= parseFloat(endSlider.value)) {
			startSlider.value = parseFloat(endSlider.value) - 0.1;
		}
		startTimeInput.value = formatTime(startSlider.value);
		updateSliderHighlight();
	});

	endSlider.addEventListener('input', () => {
		if (parseFloat(endSlider.value) <= parseFloat(startSlider.value)) {
			endSlider.value = parseFloat(startSlider.value) + 0.1;
		}
		endTimeInput.value = formatTime(endSlider.value);
		updateSliderHighlight();
	});

	startTimeInput.addEventListener('change', () => {
		let val = parseFloat(startTimeInput.value);
		if (isNaN(val) || val < 0) val = 0;
		if (val >= parseFloat(endTimeInput.value)) val = parseFloat(endTimeInput.value) - 0.1;
		startSlider.value = val;
		startTimeInput.value = formatTime(val);
		updateSliderHighlight();
	});

	endTimeInput.addEventListener('change', () => {
		let val = parseFloat(endTimeInput.value);
		const max = parseFloat(endSlider.max);
		if (isNaN(val) || val > max) val = max;
		if (val <= parseFloat(startTimeInput.value)) val = parseFloat(startTimeInput.value) + 0.1;
		endSlider.value = val;
		endTimeInput.value = formatTime(val);
		updateSliderHighlight();
	});

	conversionForm.addEventListener('submit', async (e) => {
		e.preventDefault();
		if (!videoFile) {
			alert('Please select a video file first.');
			return;
		}

		resultSection.classList.remove('hidden');
		loader.classList.remove('hidden');
		gifContainer.classList.add('hidden');
		errorContainer.classList.add('hidden');

		const formData = new FormData(conversionForm);
		formData.append('video', videoFile);

		formData.set('start_time', startTimeInput.value);
		formData.set('end_time', endTimeInput.value);

		try {
			const response = await fetch('/convert', {
				method: 'POST',
				body: formData,
			});

			if (!response.ok) {
				throw new Error(`Server responded with status: ${response.status}`);
			}

			const blob = await response.blob();
			const gifUrl = URL.createObjectURL(blob);
			gifPreview.src = gifUrl;
			downloadButton.href = gifUrl;

			loader.classList.add('hidden');
			gifContainer.classList.remove('hidden');
		} catch (error) {
			console.error('Conversion failed:', error);
			loader.classList.add('hidden');
			errorContainer.classList.remove('hidden');
		}
	});
});
