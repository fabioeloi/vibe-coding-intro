<script>
  import UploadPage from './lib/components/UploadPage.svelte';
  import SearchPage from './lib/components/SearchPage.svelte';
  import TimelinePage from './lib/components/TimelinePage.svelte';
  
  // In the future, we'll add more pages and routing here
  let currentPage = 'upload';
  
  function changePage(page) {
    currentPage = page;
  }
</script>

<main>
  <header class="app-header">
    <div class="app-logo">
      <img src="/safari-history-icon.svg" alt="Safari History Graph" />
      <h1>Safari History Graph</h1>
    </div>
    <nav class="app-nav">
      <button 
        class={currentPage === 'upload' ? 'active' : ''} 
        on:click={() => changePage('upload')}
      >
        Upload
      </button>
      <button 
        class={currentPage === 'search' ? 'active' : ''} 
        on:click={() => changePage('search')}
      >
        Search
      </button>
      <button 
        class={currentPage === 'timeline' ? 'active' : ''} 
        on:click={() => changePage('timeline')}
      >
        Timeline
      </button>
      <button 
        class={currentPage === 'dashboard' ? 'active' : ''} 
        on:click={() => changePage('dashboard')}
        disabled={currentPage === 'upload'}
      >
        Dashboard
      </button>
    </nav>
  </header>

  <div class="app-content">
    {#if currentPage === 'upload'}
      <UploadPage />
    {:else if currentPage === 'search'}
      <SearchPage />
    {:else if currentPage === 'timeline'}
      <TimelinePage />
    {:else if currentPage === 'dashboard'}
      <div class="placeholder">
        <h2>Dashboard Page</h2>
        <p>This feature will be implemented in the next phase.</p>
      </div>
    {/if}
  </div>

  <footer class="app-footer">
    <p>Safari History Knowledge Graph | AI-Powered Web History Analysis</p>
  </footer>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    background-color: #f9f9f9;
    color: #333;
  }

  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .app-header {
    background-color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 0.5rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
  }

  .app-logo {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .app-logo img {
    width: 32px;
    height: 32px;
  }

  .app-logo h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #0066cc;
  }

  .app-nav {
    display: flex;
    gap: 0.5rem;
  }

  .app-nav button {
    background: none;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    color: #555;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .app-nav button:hover:not(:disabled) {
    background-color: #f0f0f0;
    color: #333;
  }

  .app-nav button.active {
    background-color: #0066cc;
    color: white;
  }

  .app-nav button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .app-content {
    flex: 1;
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;
  }

  .app-footer {
    background-color: #f5f5f5;
    padding: 1rem 2rem;
    text-align: center;
    color: #666;
    font-size: 0.875rem;
    border-top: 1px solid #e0e0e0;
  }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    background-color: #f5f5f5;
    border-radius: 8px;
    text-align: center;
  }

  .placeholder h2 {
    margin-top: 0;
    color: #0066cc;
  }

  .placeholder p {
    color: #666;
    max-width: 500px;
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .app-header {
      flex-direction: column;
      padding: 1rem;
    }

    .app-logo {
      margin-bottom: 0.5rem;
    }

    .app-nav {
      width: 100%;
      justify-content: space-between;
    }

    .app-nav button {
      padding: 0.5rem;
    }

    .app-content {
      padding: 1rem;
    }
  }
</style>
