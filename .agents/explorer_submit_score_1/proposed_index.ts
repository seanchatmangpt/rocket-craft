// deno-lint-ignore-file no-import-prefix
import { serve } from "https://deno.land/std@0.168.0/http/server.ts"
import { createClient } from "https://esm.sh/@supabase/supabase-js@2"

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Headers': 'authorization, x-client-info, apikey, content-type',
}

interface ScoreRequest {
  score: number;
}

serve(async (req) => {
  // Handle CORS preflight request
  if (req.method === 'OPTIONS') {
    return new Response('ok', {
      headers: corsHeaders,
    });
  }

  try {
    // 1. Extract Authorization header and JWT token
    const authHeader = req.headers.get("Authorization") || req.headers.get("authorization");
    if (!authHeader || !authHeader.startsWith("Bearer ")) {
      return new Response(
        JSON.stringify({ error: "Missing or invalid Authorization header" }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 401,
        }
      );
    }
    const token = authHeader.substring(7);

    // 2. Parse and validate the score
    const { score }: ScoreRequest = await req.json();

    if (
      typeof score !== 'number' ||
      isNaN(score) ||
      !Number.isInteger(score) ||
      score < 0 ||
      score > 1000
    ) {
      return new Response(
        JSON.stringify({ error: 'Invalid score. Score must be a number between 0 and 1000.' }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 400,
        }
      );
    }

    // 3. Construct Supabase client and query the user
    const supabaseUrl = Deno.env.get("SUPABASE_URL");
    const supabaseAnonKey = Deno.env.get("SUPABASE_ANON_KEY");

    if (!supabaseUrl || !supabaseAnonKey) {
      return new Response(
        JSON.stringify({ error: "Server configuration error: missing Supabase credentials." }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 500,
        }
      );
    }

    const supabaseClient = createClient(supabaseUrl, supabaseAnonKey, {
      global: {
        headers: {
          Authorization: authHeader,
        },
      },
    });

    const { data: { user }, error: userError } = await supabaseClient.auth.getUser(token);
    if (userError || !user) {
      return new Response(
        JSON.stringify({ error: "Missing or invalid Authorization header" }),
        {
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
          status: 401,
        }
      );
    }

    // 4. Save score to public.game_sessions
    const { error: sessionError } = await supabaseClient
      .from("game_sessions")
      .insert({
        player_id: user.id,
        score: score,
      });

    if (sessionError) {
      throw new Error(`Failed to save game session: ${sessionError.message}`);
    }

    // 5. Query public.leaderboard for existing entry
    const { data: leaderboardEntry, error: queryError } = await supabaseClient
      .from("leaderboard")
      .select("id, score")
      .eq("player_id", user.id)
      .maybeSingle();

    if (queryError) {
      throw new Error(`Failed to query leaderboard: ${queryError.message}`);
    }

    if (!leaderboardEntry) {
      // 5a. Insert new leaderboard entry if none exists
      const { error: insertError } = await supabaseClient
        .from("leaderboard")
        .insert({
          player_id: user.id,
          score: score,
        });
      
      if (insertError) {
        throw new Error(`Failed to insert into leaderboard: ${insertError.message}`);
      }
    } else if (score > leaderboardEntry.score) {
      // 5b. Update leaderboard if the new score is higher than the existing high score
      const { error: updateError } = await supabaseClient
        .from("leaderboard")
        .update({
          score: score,
        })
        .eq("id", leaderboardEntry.id);
      
      if (updateError) {
        throw new Error(`Failed to update leaderboard: ${updateError.message}`);
      }
    }

    // 6. Return success response
    return new Response(
      JSON.stringify({
        message: `Score of ${score} submitted successfully!`,
        score: score,
      }),
      {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        status: 200,
      }
    );
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return new Response(
      JSON.stringify({ error: errorMessage }),
      {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        status: 500,
      }
    );
  }
});
