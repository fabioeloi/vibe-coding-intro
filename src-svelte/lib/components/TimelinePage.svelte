<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  // Timeline state
  let timelineData = [];
  let isLoading = true;
  let error = '';
  let timeRange = 'week';
  let startDate = '';
  let endDate = '';
  let selectedDomain = '';
  let domains = [];
  let groupBy = 'day'; // day, hour, domain
  
  // Time range options
  const timeRanges = [
    { value: 'day', label: 'Last 24 Hours' },
    { value: 'week', label: 'Last 7 Days' },
    { value: 'month', label: 'Last 30 Days' },
    { value: 'quarter', label: 'Last 90 Days' },
    { value: 'year', label: 'Last Year' },
    { value: 'all', label: 'All Time' },
    { value: 'custom', label: 'Custom Range' }
  ];
  
  // Group by options
  const groupByOptions = [
    { value: 'hour', label: 'Hour' },
    { value: 'day', label: 'Day' },
    { value: 'domain', label: 'Domain' }
  ];
  
  // Day labels for the timeline
  const dayLabels = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  
  // Hour labels
  const hourLabels = Array.from({ length: 24 }, (_, i) => `${i}:00`);
  
  // Color scale for visualization
  const colorScale = [
    '#f7fbff', '#deebf7', '#c6dbef', '#9ecae1', 
    '#6baed6', '#4292c6', '#2171b5', '#08519c', '#08306b'
  ];
  
  // Initialize component
  onMount(async () => {
    try {
      // Set date range based on the default selection
      setTimeRange(timeRange);
      
      // Fetch domains for filter
      domains = await fetchDomains();
      
      // Load initial timeline data
      await loadTimelineData();
    } catch (err) {
      error = `Failed to initialize timeline: ${err.message}`;
      isLoading = false;
    }
  });
  
  // Set the time range based on selection
  function setTimeRange(range) {
    timeRange = range;
    const now = new Date();
    let start = new Date();
    
    switch (range) {
      case 'day':
        start.setDate(start.getDate() - 1);
        break;
      case 'week':
        start.setDate(start.getDate() - 7);
        break;
      case 'month':
        start.setDate(start.getDate() - 30);
        break;
      case 'quarter':
        start.setDate(start.getDate() - 90);
        break;
      case 'year':
        start.setDate(start.getDate() - 365);
        break;
      case 'all':
        start = null;
        break;
      case 'custom':
        // Don't change dates for custom range
        return;
    }
    
    startDate = start ? start.toISOString().split('T')[0] : '';
    endDate = range !== 'all' ? now.toISOString().split('T')[0] : '';
    
    // Reload data if not in custom mode
    if (range !== 'custom') {
      loadTimelineData();
    }
  }
  
  // Apply custom date range
  function applyCustomDateRange() {
    if (startDate && endDate) {
      loadTimelineData();
    }
  }
  
  // Fetch domains for the filter
  async function fetchDomains() {
    try {
      const stats = await invoke('get_history_stats');
      return stats.top_domains || [];
    } catch (err) {
      console.error('Failed to fetch domains:', err);
      return [];
    }
  }
  
  // Load timeline data based on current filters
  async function loadTimelineData() {
    isLoading = true;
    error = '';
    
    try {
      // Format dates for backend
      const formattedStartDate = startDate ? new Date(startDate).toISOString() : null;
      const formattedEndDate = endDate ? new Date(`${endDate}T23:59:59`).toISOString() : null;
      
      // Fetch timeline data from backend
      const data = await invoke('get_timeline_data', {
        startDate: formattedStartDate,
        endDate: formattedEndDate,
        domain: selectedDomain || null,
        groupBy: groupBy
      });
      
      // Process data for visualization
      timelineData = processTimelineData(data);
    } catch (err) {
      error = `Failed to load timeline data: ${err.message}`;
      timelineData = [];
    } finally {
      isLoading = false;
    }
  }
  
  // Process raw timeline data for visualization
  function processTimelineData(data) {
    if (!data || data.length === 0) return [];
    
    if (groupBy === 'hour') {
      // Process hourly data
      return processHourlyData(data);
    } else if (groupBy === 'day') {
      // Process daily data
      return processDailyData(data);
    } else if (groupBy === 'domain') {
      // Process domain data
      return processDomainData(data);
    }
    
    return data;
  }
  
  // Process hourly data for visualization
  function processHourlyData(data) {
    // Create an array for each hour
    const hourlyData = Array(24).fill().map((_, i) => ({
      hour: i,
      label: `${i}:00`,
      count: 0,
      urls: []
    }));
    
    // Fill with actual data
    data.forEach(item => {
      const hour = new Date(item.timestamp).getHours();
      hourlyData[hour].count += item.count;
      if (item.urls) hourlyData[hour].urls.push(...item.urls);
    });
    
    return hourlyData;
  }
  
  // Process daily data for visualization
  function processDailyData(data) {
    // Get date range
    let start, end;
    if (startDate) {
      start = new Date(startDate);
    } else {
      // Default to last 7 days if no start date
      start = new Date();
      start.setDate(start.getDate() - 7);
    }
    
    if (endDate) {
      end = new Date(endDate);
    } else {
      end = new Date();
    }
    
    // Create array for each day in range
    const days = [];
    let current = new Date(start);
    while (current <= end) {
      days.push({
        date: new Date(current),
        dateStr: current.toISOString().split('T')[0],
        dayOfWeek: current.getDay(),
        label: dayLabels[current.getDay()],
        count: 0,
        urls: []
      });
      current.setDate(current.getDate() + 1);
    }
    
    // Fill with actual data
    data.forEach(item => {
      const dateStr = new Date(item.timestamp).toISOString().split('T')[0];
      const dayIndex = days.findIndex(d => d.dateStr === dateStr);
      if (dayIndex >= 0) {
        days[dayIndex].count += item.count;
        if (item.urls) days[dayIndex].urls.push(...item.urls);
      }
    });
    
    return days;
  }
  
  // Process domain data for visualization
  function processDomainData(data) {
    // Sort by count descending and take top 20
    return data
      .sort((a, b) => b.count - a.count)
      .slice(0, 20)
      .map(item => ({
        domain: item.domain,
        label: item.domain,
        count: item.count,
        urls: item.urls || []
      }));
  }
  
  // Get color intensity based on count
  function getColorIntensity(count, max) {
    if (max === 0) return colorScale[0];
    const index = Math.min(Math.floor((count / max) * (colorScale.length - 1)), colorScale.length - 1);
    return colorScale[index];
  }
  
  // Get the maximum count in the current dataset
  function getMaxCount() {
    if (!timelineData || timelineData.length === 0) return 0;
    return Math.max(...timelineData.map(d => d.count));
  }
  
  // Format time for display
  function formatTime(timestamp) {
    return new Date(timestamp).toLocaleString();
  }
  
  // Apply current filters and reload data
  function applyFilters() {
    loadTimelineData();
  }
  
  // Reset all filters
  function resetFilters() {
    timeRange = 'week';
    setTimeRange(timeRange);
    selectedDomain = '';
    groupBy = 'day';
    loadTimelineData();
  }
</script>

<div class="timeline-page">
  <h1>Timeline View</h1>
  <p class="description">Visualize your browsing activity over time to discover patterns and trends.</p>
  
  <div class="filters">
    <div class="filter-section">
      <label for="timeRange">Time Range</label>
      <select id="timeRange" bind:value={timeRange} on:change={() => setTimeRange(timeRange)}>
        {#each timeRanges as range}
          <option value={range.value}>{range.label}</option>
        {/each}
      </select>
    </div>
    
    {#if timeRange === 'custom'}
      <div class="custom-date-range">
        <div class="date-input">
          <label for="startDate">From</label>
          <input type="date" id="startDate" bind:value={startDate}>
        </div>
        <div class="date-input">
          <label for="endDate">To</label>
          <input type="date" id="endDate" bind:value={endDate}>
        </div>
        <button class="apply-button" on:click={applyCustomDateRange}>Apply</button>
      </div>
    {/if}
    
    <div class="filter-section">
      <label for="groupBy">Group By</label>
      <select id="groupBy" bind:value={groupBy} on:change={applyFilters}>
        {#each groupByOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    
    <div class="filter-section">
      <label for="domain">Domain</label>
      <select id="domain" bind:value={selectedDomain} on:change={applyFilters}>
        <option value="">All Domains</option>
        {#each domains as domain}
          <option value={domain[0]}>{domain[0]}</option>
        {/each}
      </select>
    </div>
    
    <button class="reset-button" on:click={resetFilters}>Reset Filters</button>
  </div>
  
  {#if error}
    <div class="error-message">
      <p>{error}</p>
    </div>
  {/if}
  
  {#if isLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading timeline data...</p>
    </div>
  {:else if timelineData.length === 0}
    <div class="empty-state">
      <h3>No data available</h3>
      <p>No browsing activity found for the selected time range and filters.</p>
      <button class="reset-button" on:click={resetFilters}>Reset Filters</button>
    </div>
  {:else}
    <div class="timeline-container">
      <h2>Browsing Activity {groupBy === 'hour' ? 'by Hour' : groupBy === 'day' ? 'by Day' : 'by Domain'}</h2>
      
      <div class="visualization">
        {#if groupBy === 'hour'}
          <!-- Hourly view -->
          <div class="hour-view">
            {#each timelineData as hourData}
              <div 
                class="hour-bar" 
                style="height: {Math.max(5, (hourData.count / getMaxCount()) * 150)}px; background-color: {getColorIntensity(hourData.count, getMaxCount())}"
                title="{hourData.count} visits at {hourData.label}"
              >
                <div class="hour-count">{hourData.count}</div>
              </div>
            {/each}
          </div>
          <div class="hour-labels">
            {#each hourLabels as label, i}
              {#if i % 2 === 0}
                <div class="hour-label">{label}</div>
              {:else}
                <div class="hour-label empty"></div>
              {/if}
            {/each}
          </div>
        {:else if groupBy === 'day'}
          <!-- Daily view -->
          <div class="day-view">
            {#each timelineData as dayData}
              <div class="day-column">
                <div 
                  class="day-bar" 
                  style="height: {Math.max(5, (dayData.count / getMaxCount()) * 150)}px; background-color: {getColorIntensity(dayData.count, getMaxCount())}"
                  title="{dayData.count} visits on {dayData.dateStr}"
                >
                  <div class="day-count">{dayData.count}</div>
                </div>
                <div class="day-label">{dayData.label}</div>
                <div class="day-date">{dayData.dateStr.slice(5)}</div>
              </div>
            {/each}
          </div>
        {:else if groupBy === 'domain'}
          <!-- Domain view -->
          <div class="domain-view">
            {#each timelineData as domainData, i}
              <div class="domain-row">
                <div class="domain-name">{domainData.label}</div>
                <div 
                  class="domain-bar" 
                  style="width: {Math.max(5, (domainData.count / getMaxCount()) * 300)}px; background-color: {getColorIntensity(domainData.count, getMaxCount())}"
                  title="{domainData.count} visits to {domainData.domain}"
                >
                  <div class="domain-count">{domainData.count}</div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      
      <!-- Activity details -->
      <div class="activity-details">
        <h3>Most Visited Pages</h3>
        <div class="url-list">
          {#if groupBy === 'domain'}
            <!-- Show top pages for selected domain -->
            {#if timelineData.length > 0 && timelineData[0].urls && timelineData[0].urls.length > 0}
              <ul>
                {#each timelineData[0].urls.slice(0, 10) as url}
                  <li>
                    <a href={url.url} target="_blank" rel="noopener noreferrer">
                      {url.title || url.url}
                    </a>
                    <span class="visit-count">{url.visit_count} visits</span>
                    <span class="visit-time">Last: {formatTime(url.last_visit || url.last_seen)}</span>
                  </li>
                {/each}
              </ul>
            {:else}
              <p>No detailed URL data available for the selected view.</p>
            {/if}
          {:else}
            <!-- Combined all URLs in the current view -->
            {#if timelineData.some(d => d.urls && d.urls.length > 0)}
              <ul>
                {#each timelineData
                  .flatMap(d => d.urls || [])
                  .sort((a, b) => b.visit_count - a.visit_count)
                  .slice(0, 10) as url}
                  <li>
                    <a href={url.url} target="_blank" rel="noopener noreferrer">
                      {url.title || url.url}
                    </a>
                    <span class="visit-count">{url.visit_count} visits</span>
                    <span class="visit-time">Last: {formatTime(url.last_visit || url.last_seen)}</span>
                  </li>
                {/each}
              </ul>
            {:else}
              <p>No detailed URL data available for the selected view.</p>
            {/if}
          {/if}
        </div>
      </div>
    </div>
  {/if}
  
  <div class="timeline-hint">
    <h3>Timeline Tips</h3>
    <ul>
      <li>Use the <strong>Time Range</strong> filter to focus on specific periods</li>
      <li>Switch between <strong>Hour</strong>, <strong>Day</strong>, and <strong>Domain</strong> views to see different patterns</li>
      <li>Filter by <strong>Domain</strong> to analyze your browsing habits on specific websites</li>
      <li>Taller/wider bars indicate more browsing activity in that period or domain</li>
    </ul>
  </div>
</div>

<style>
  .timeline-page {
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }
  
  h1 {
    font-size: 2rem;
    margin-bottom: 0.5rem;
    color: #333;
  }
  
  .description {
    color: #666;
    margin-bottom: 2rem;
  }
  
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    background-color: #f5f5f5;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
    align-items: flex-end;
  }
  
  .filter-section {
    display: flex;
    flex-direction: column;
    min-width: 150px;
  }
  
  .filter-section label {
    font-size: 0.9rem;
    margin-bottom: 0.25rem;
    color: #555;
  }
  
  .filter-section select {
    padding: 0.65rem 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
    background-color: white;
  }
  
  .custom-date-range {
    display: flex;
    gap: 0.5rem;
    align-items: flex-end;
  }
  
  .date-input {
    display: flex;
    flex-direction: column;
  }
  
  .date-input label {
    font-size: 0.9rem;
    margin-bottom: 0.25rem;
    color: #555;
  }
  
  .date-input input {
    padding: 0.65rem 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
    background-color: white;
  }
  
  .apply-button, .reset-button {
    padding: 0.65rem 1.25rem;
    border: none;
    border-radius: 4px;
    font-weight: 500;
    cursor: pointer;
    font-size: 1rem;
    transition: background-color 0.2s ease;
  }
  
  .apply-button {
    background-color: #4d4dff;
    color: white;
  }
  
  .apply-button:hover {
    background-color: #3a3abf;
  }
  
  .reset-button {
    background-color: #e0e0e0;
    color: #555;
  }
  
  .reset-button:hover {
    background-color: #d0d0d0;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
    color: #666;
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 3px solid #f3f3f3;
    border-top: 3px solid #4d4dff;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .error-message {
    background-color: #fff0f0;
    border: 1px solid #ffcccc;
    border-radius: 4px;
    padding: 1rem;
    margin-bottom: 1.5rem;
    color: #cc0000;
  }
  
  .empty-state {
    text-align: center;
    padding: 3rem;
    background-color: #f9f9f9;
    border-radius: 8px;
    color: #666;
  }
  
  .empty-state h3 {
    margin-top: 0;
    color: #333;
  }
  
  .empty-state button {
    margin-top: 1rem;
  }
  
  .timeline-container {
    background-color: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.05);
    margin-bottom: 2rem;
  }
  
  .timeline-container h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    color: #333;
    font-size: 1.5rem;
  }
  
  .visualization {
    margin-bottom: 2rem;
    overflow-x: auto;
  }
  
  /* Hour view */
  .hour-view {
    display: flex;
    align-items: flex-end;
    height: 170px;
    padding-bottom: 10px;
    gap: 12px;
  }
  
  .hour-bar {
    width: 28px;
    border-radius: 4px 4px 0 0;
    position: relative;
    transition: all 0.2s ease;
  }
  
  .hour-bar:hover {
    transform: translateY(-5px);
    box-shadow: 0 5px 10px rgba(0,0,0,0.1);
  }
  
  .hour-count {
    position: absolute;
    bottom: -20px;
    left: 0;
    right: 0;
    text-align: center;
    font-size: 0.8rem;
    color: #666;
  }
  
  .hour-labels {
    display: flex;
    gap: 12px;
    margin-top: 20px;
  }
  
  .hour-label {
    width: 28px;
    text-align: center;
    font-size: 0.8rem;
    color: #666;
  }
  
  .hour-label.empty {
    color: transparent;
  }
  
  /* Day view */
  .day-view {
    display: flex;
    align-items: flex-end;
    height: 200px;
    padding-bottom: 10px;
    gap: 12px;
    overflow-x: auto;
  }
  
  .day-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 50px;
  }
  
  .day-bar {
    width: 30px;
    border-radius: 4px 4px 0 0;
    position: relative;
    transition: all 0.2s ease;
  }
  
  .day-bar:hover {
    transform: translateY(-5px);
    box-shadow: 0 5px 10px rgba(0,0,0,0.1);
  }
  
  .day-count {
    position: absolute;
    bottom: -20px;
    left: 0;
    right: 0;
    text-align: center;
    font-size: 0.8rem;
    color: #666;
  }
  
  .day-label {
    margin-top: 20px;
    font-size: 0.9rem;
    font-weight: 500;
    color: #333;
  }
  
  .day-date {
    font-size: 0.8rem;
    color: #666;
    margin-top: 4px;
  }
  
  /* Domain view */
  .domain-view {
    display: flex;
    flex-direction: column;
    gap: 15px;
    max-height: 500px;
    overflow-y: auto;
  }
  
  .domain-row {
    display: flex;
    align-items: center;
    gap: 15px;
  }
  
  .domain-name {
    width: 150px;
    font-size: 0.9rem;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .domain-bar {
    height: 25px;
    border-radius: 4px;
    position: relative;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    min-width: 30px;
  }
  
  .domain-bar:hover {
    transform: translateX(5px);
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
  }
  
  .domain-count {
    padding: 0 10px;
    font-size: 0.85rem;
    color: #333;
    font-weight: 500;
    white-space: nowrap;
  }
  
  /* Activity details */
  .activity-details {
    margin-top: 2rem;
    border-top: 1px solid #eee;
    padding-top: 1.5rem;
  }
  
  .activity-details h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #333;
  }
  
  .url-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .url-list li {
    padding: 0.75rem;
    border-bottom: 1px solid #f0f0f0;
    display: flex;
    flex-direction: column;
  }
  
  .url-list li:last-child {
    border-bottom: none;
  }
  
  .url-list a {
    color: #0066cc;
    text-decoration: none;
    font-weight: 500;
    margin-bottom: 0.25rem;
    word-break: break-all;
  }
  
  .url-list a:hover {
    text-decoration: underline;
  }
  
  .visit-count, .visit-time {
    font-size: 0.85rem;
    color: #666;
  }
  
  .visit-count {
    margin-right: 1rem;
  }
  
  /* Timeline hint */
  .timeline-hint {
    background-color: #f0f8ff;
    padding: 1.5rem;
    border-radius: 8px;
    border-left: 4px solid #4d4dff;
  }
  
  .timeline-hint h3 {
    margin-top: 0;
    margin-bottom: 0.75rem;
    color: #333;
  }
  
  .timeline-hint ul {
    margin: 0;
    padding-left: 1.25rem;
  }
  
  .timeline-hint li {
    margin-bottom: 0.5rem;
    color: #555;
  }
  
  .timeline-hint strong {
    color: #333;
  }
  
  /* Responsive adjustments */
  @media (max-width: 768px) {
    .filters {
      flex-direction: column;
      align-items: stretch;
    }
    
    .custom-date-range {
      flex-direction: column;
    }
    
    .domain-name {
      width: 100px;
    }
    
    .hour-view, .day-view {
      padding-left: 10px;
      padding-right: 10px;
    }
  }
</style>
