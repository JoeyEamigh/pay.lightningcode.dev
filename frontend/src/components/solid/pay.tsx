import { ErrorBoundary, Show, createSignal } from 'solid-js';
import { GetInvoice } from './invoice';
import { StripeHandler } from './stripe';

export const api = import.meta.env.PUBLIC_API_URL;

export type Invoice = {
  id: string;
  pdfUrl: string;
  invoiceNumber: string;
  customer: {
    name: string;
  };
  amountDue: {
    value: number;
  };
};

const getFee = (amount: number): number => (Math.round((amount + 30) / (1 - 0.029)) - amount) / 100;

export function PayInvoice(props: { mode: 'card' | 'ach' }) {
  const [invoice, setInvoice] = createSignal<Invoice | null>(null);

  return (
    <ErrorBoundary fallback={err => <ErrorFallback error={err} />}>
      <Show when={invoice()} fallback={<GetInvoice setInvoice={setInvoice} />}>
        {invoice => <Review invoice={invoice()} mode={props.mode} />}
      </Show>
    </ErrorBoundary>
  );
}

function ErrorFallback(props: { error: Error }) {
  return (
    <section class="px-4">
      <h2 class="text-center text-3xl">oh no :&#40;</h2>
      <p class="mx-auto mt-2 max-w-prose text-center text-xl">{props.error.message}</p>
    </section>
  );
}

function Review(props: { invoice: Invoice; mode: 'card' | 'ach' }) {
  const [showForm, setShowForm] = createSignal(false);

  return (
    <section class="px-4">
      <p class="text-center text-xl">{props.invoice.customer.name}</p>
      <h2 class="text-center text-3xl">Invoice #{props.invoice.invoiceNumber}</h2>
      <p class="mt-2 text-center text-2xl">
        Amount Due:{' '}
        {(props.invoice.amountDue.value / 100).toLocaleString('en-US', {
          style: 'currency',
          currency: 'USD',
        })}{' '}
        <Show when={props.mode === 'card'}>
          +{' '}
          {getFee(props.invoice.amountDue.value).toLocaleString('en-US', {
            style: 'currency',
            currency: 'USD',
          })}{' '}
          fee
        </Show>
      </p>
      <div class="mt-8 flex justify-center gap-4">
        <a
          target="_blank"
          rel="noopener noreferrer"
          href={props.invoice.pdfUrl}
          class="inline-flex items-center rounded-md border border-gray-200 bg-gray-800 px-4 py-2 text-sm font-medium text-gray-100 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2">
          Download Invoice
        </a>
        {!showForm() && (
          <button
            onClick={() => setShowForm(true)}
            type="button"
            class="inline-flex items-center rounded-md border border-gray-200 bg-gray-800 px-4 py-2 text-sm font-medium text-gray-100 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2">
            Pay Invoice
          </button>
        )}
      </div>

      <Show when={showForm()}>
        <StripeHandler invoice={props.invoice} mode={props.mode} />
      </Show>
    </section>
  );
}
