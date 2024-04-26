import { createSignal, type JSX, type Setter } from 'solid-js';
import { api, type Invoice } from './pay';

const checkInvoice = async (id: string | null): Promise<Invoice | null> => {
  if (!id) throw new Error('invoice not found');

  const res = await fetch(`${api}/invoice/${id}`);
  if (!res.ok) throw new Error('invoice not found');

  return res.json();
};

export function GetInvoice(props: { setInvoice: Setter<Invoice | null> }) {
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const handleSubmit: JSX.EventHandler<HTMLFormElement, SubmitEvent> = async e => {
    e.preventDefault();
    setError(null);

    if (loading()) return;

    const formData = new FormData(e.currentTarget);
    const id = formData.get('invoiceId');
    if (!id || !id.length || typeof id !== 'string') return;

    setLoading(true);
    try {
      const invoice = await checkInvoice(id);
      props.setInvoice(invoice);
    } catch (err: unknown) {
      if (err instanceof Error) setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <section class="px-4">
      <div class="mx-auto max-w-xs">
        <form onSubmit={handleSubmit} class="px-8 lg:px-16">
          <label for="invoiceId" class="block text-center text-sm font-medium text-gray-300">
            Invoice number
          </label>
          <div class="relative mt-1 rounded-md shadow-sm">
            <input
              type="text"
              name="invoiceId"
              id="invoiceId"
              class="block w-full rounded-md border border-gray-300 bg-gray-800 px-3 py-2 !text-base text-white focus:border-gray-100 focus:ring-gray-100 sm:text-sm"
              placeholder="0000"
            />
          </div>
          <p class="mt-4 text-center text-sm text-red-500 empty:hidden">{error()}</p>
          <div class="mt-4 text-center">
            <button
              type="submit"
              disabled={loading()}
              class="inline-flex items-center rounded-md border border-gray-200 bg-gray-800 px-4 py-2 text-sm font-medium text-gray-100 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2">
              {loading() ? 'Searching...' : 'Find Invoice'}
            </button>
          </div>
        </form>
      </div>
    </section>
  );
}
