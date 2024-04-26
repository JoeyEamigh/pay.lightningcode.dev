import type { Stripe } from '@stripe/stripe-js';
import { loadStripe } from '@stripe/stripe-js/pure';
import { Show, Suspense, createResource, createSignal, type JSX } from 'solid-js';
import Skeleton from 'solid-loading-skeleton';
import { Elements, PaymentElement, useStripeProxy } from 'solid-stripe';
import { api, type Invoice } from './pay';

const stripePublicKey = import.meta.env.PUBLIC_STRIPE_PUBLIC_KEY;

type PaymentIntentRes = { clientSecret: string };

const createPaymentIntent = async (invoiceId: string, mode: string): Promise<string> => {
  const res = await fetch(`${api}/invoice/${invoiceId}/pay/${mode}`);
  if (!res.ok) return Promise.reject(new Error('something went wrong'));

  const body: PaymentIntentRes = await res.json();
  return body.clientSecret;
};

export function StripeHandler(props: { invoice: Invoice; mode: 'card' | 'ach' }) {
  const [stripe] = createResource<Stripe | null>(() => loadStripe(stripePublicKey));
  const [clientSecret] = createResource<string | null>(() => createPaymentIntent(props.invoice.id, props.mode));

  return (
    <Suspense fallback={<Skeleton count={10} />}>
      <Show when={stripe() && clientSecret()}>
        {clientSecret => (
          <Elements stripe={stripe() || null} clientSecret={clientSecret()} theme="night">
            <StripeForm />
          </Elements>
        )}
      </Show>
    </Suspense>
  );
}

function StripeForm() {
  const { stripe, elements } = useStripeProxy();
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [stripeLoading, setStripeLoading] = createSignal(true);
  const [loading, setLoading] = createSignal(false);

  const submit: JSX.EventHandler<HTMLFormElement, SubmitEvent> = async e => {
    e?.preventDefault?.();
    if (!stripe || !elements || loading()) return;
    setLoading(true);

    setErrorMessage(null);
    const result = await stripe.confirmPayment({
      elements,
      confirmParams: { return_url: `${api}/stripe/callback` },
    });

    if (result.error) {
      setErrorMessage(result.error.message || '');
      setLoading(false);
    }
  };

  return (
    <form onSubmit={submit} class="mx-auto mb-16 mt-8 max-w-xl">
      <PaymentElement onReady={() => setStripeLoading(false)} />
      <Show when={stripeLoading()}>
        <Skeleton count={10} />
      </Show>
      <p class="mt-4 text-center text-sm text-red-600 empty:hidden">{errorMessage()}</p>
      <div class="flex flex-wrap justify-center gap-4 pt-4">
        <button
          type="submit"
          disabled={loading()}
          class="inline-flex items-center rounded-md border border-gray-200 bg-gray-800 px-4 py-2 text-sm font-medium text-gray-100 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2">
          {loading() ? 'Processing...' : 'Pay Invoice'}
        </button>
      </div>
    </form>
  );
}
