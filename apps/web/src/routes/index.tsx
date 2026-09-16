import { createFileRoute } from "@tanstack/react-router";

function Home() {
	return (
		<main className="flex min-h-dvh items-center justify-center px-4">
			<h1 className="text-3xl font-bold tracking-tight sm:text-5xl">
				Clinicore
			</h1>
		</main>
	);
}

export const Route = createFileRoute("/")({
	component: Home,
});
