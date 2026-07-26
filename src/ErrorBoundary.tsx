import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  failed: boolean;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  state: ErrorBoundaryState = { failed: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    if (import.meta.env.DEV) {
      console.error("React rendering failed", error.name, info.componentStack);
    }
  }

  render() {
    if (this.state.failed) {
      return (
        <main className="flex min-h-screen items-center justify-center bg-stone-950 p-8 text-stone-100">
          <div className="max-w-md rounded-xl border border-red-900 bg-red-950/30 p-6">
            <h1 className="text-xl font-semibold text-red-200">
              Book Library needs to restart
            </h1>
            <p className="mt-3 text-sm leading-6 text-red-300">
              The interface could not be displayed. Your source books were not
              changed.
            </p>
          </div>
        </main>
      );
    }

    return this.props.children;
  }
}
