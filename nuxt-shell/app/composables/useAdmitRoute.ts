/**
 * useAdmitRoute — route admission guard composable.
 *
 * Port of ~/expo-supabase-ai-template/src/framework/auth/guards.ts (admitRoute)
 * to Vue 3 / Nuxt 4. The original is React / React Native; the logic is identical
 * but adapted to composable conventions and Supabase auth.
 *
 * Identity hierarchy (ordered, ascending privilege):
 *   anonymous → authenticated → verified → mfa_verified
 *
 * Usage in route middleware:
 *   const { admitted, refusal } = useAdmitRoute({ requiredIdentityBoundary: 'authenticated' });
 *   if (!admitted) return navigateTo(`/?reason=${refusal?.code}`);
 *
 * Usage in components for conditional rendering:
 *   const { admitted } = useAdmitRoute({ requiredIdentityBoundary: 'authenticated' });
 *   // <div v-if="admitted">protected content</div>
 */

export type IdentityBoundary = 'anonymous' | 'authenticated' | 'verified' | 'mfa_verified';

export interface RouteDefinition {
  requiredIdentityBoundary?: IdentityBoundary;
  requiredRoles?: string[];
  requiredPermissions?: string[];
}

export interface RefusalReason {
  code: string;
  message: string;
  requiredIdentityBoundary?: IdentityBoundary;
  actualIdentityBoundary?: IdentityBoundary;
  missingRoles?: string[];
  missingPermissions?: string[];
}

export interface AdmitRouteResult {
  admitted: boolean;
  refusal?: RefusalReason;
}

const IDENTITY_HIERARCHY: readonly IdentityBoundary[] = [
  'anonymous',
  'authenticated',
  'verified',
  'mfa_verified',
];

function checkRoute(
  identityBoundary: IdentityBoundary,
  route: RouteDefinition,
  roles: string[],
): AdmitRouteResult {
  if (route.requiredIdentityBoundary) {
    const requiredIdx = IDENTITY_HIERARCHY.indexOf(route.requiredIdentityBoundary);
    const actualIdx = IDENTITY_HIERARCHY.indexOf(identityBoundary);

    if (identityBoundary === 'anonymous' && route.requiredIdentityBoundary !== 'anonymous') {
      return {
        admitted: false,
        refusal: {
          code: 'UNAUTHENTICATED',
          message: 'Authentication required.',
          requiredIdentityBoundary: route.requiredIdentityBoundary,
          actualIdentityBoundary: identityBoundary,
        },
      };
    }
    if (actualIdx < requiredIdx) {
      return {
        admitted: false,
        refusal: {
          code: 'INSUFFICIENT_IDENTITY_LEVEL',
          message: `Level "${identityBoundary}" is insufficient. Required: "${route.requiredIdentityBoundary}".`,
          requiredIdentityBoundary: route.requiredIdentityBoundary,
          actualIdentityBoundary: identityBoundary,
        },
      };
    }
  }

  if (route.requiredRoles?.length) {
    const roleSet = new Set(roles);
    const missing = route.requiredRoles.filter(r => !roleSet.has(r));
    if (missing.length) {
      return {
        admitted: false,
        refusal: { code: 'MISSING_ROLE', message: `Missing role(s): ${missing.join(', ')}`, missingRoles: missing },
      };
    }
  }

  return { admitted: true };
}

export function useAdmitRoute(route: RouteDefinition): { admitted: ComputedRef<boolean>; refusal: ComputedRef<RefusalReason | undefined> } {
  const { user } = useRocketSupabase();

  const result = computed<AdmitRouteResult>(() => {
    const identity: IdentityBoundary = user.value ? 'authenticated' : 'anonymous';
    // Roles could be stored in user_metadata — extend here if MFA/roles are added
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const roles: string[] = ((user.value as any)?.user_metadata?.roles as string[] | undefined) ?? [];
    return checkRoute(identity, route, roles);
  });

  return {
    admitted: computed(() => result.value.admitted),
    refusal: computed(() => result.value.refusal),
  };
}
