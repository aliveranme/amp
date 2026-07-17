export async function generateStaticParams() {
  return [{ id: "index" }];
}

export default function UserLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
